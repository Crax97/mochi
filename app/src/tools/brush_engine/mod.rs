pub mod stamping_engine;
pub mod stamping_engine_pass;

use cgmath::{InnerSpace, Point2};
use wgpu::CommandEncoder;

use image_editor::{layers::Layer, ImageEditor};

use super::{EditorCommand, EditorContext};

#[derive(Debug)]
pub struct StrokePoint {
    pub position: Point2<f32>,
    pub size: f32,
}

#[derive(Debug)]
pub struct StrokePath {
    pub points: Vec<StrokePoint>,
}

impl std::fmt::Display for StrokePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("StrokePath {\n")?;
        f.write_str("\tpoints: [\n")?;
        for pt in self.points.iter() {
            f.write_str(format!("\t\t{:?},\n", pt).as_str())?;
        }
        f.write_str("\t],\n")?;
        f.write_str("}")?;

        Ok(())
    }
}

pub struct StrokeContext<'editor, 'stroke> {
    pub layer: &'editor Layer<'editor>,
    pub editor: &'stroke ImageEditor<'editor>,
    pub command_encoder: &'stroke mut CommandEncoder,
}

pub trait BrushEngine {
    fn stroke(&mut self, path: StrokePath, context: StrokeContext);
    fn end_stroking(&mut self, context: &mut EditorContext) -> Option<Box<dyn EditorCommand>>;
}
impl StrokePath {
    pub(crate) fn linear_start_to_end(start: StrokePoint, end: StrokePoint, step: f32) -> Self {
        let direction = end.position - start.position;
        let distance = direction.magnitude();
        let direction = direction.normalize();
        let size_delta = end.size - start.size;
        let num_points = (distance / step) as usize;
        let points = (0..num_points)
            .into_iter()
            .map(|pt| {
                let distance_in_path = pt as f32 * step;
                let position = start.position + distance_in_path * direction;
                let size = start.size + size_delta * (distance_in_path / distance);
                StrokePoint { position, size }
            })
            .chain(std::iter::once(end))
            .collect();
        StrokePath { points }
    }
}
