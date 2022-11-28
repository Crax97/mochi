mod stamp_operation;
pub mod stamping_engine;

use cgmath::{vec2, InnerSpace, Point2};

use framework::{renderer::renderer::Renderer, Box2d, Framework};
use image_editor::ImageEditor;

use super::{EditorCommand, EditorContext};

#[derive(Debug)]
pub struct StrokePoint {
    pub position: Point2<f32>,
    pub size: f32,
}

#[derive(Debug)]
pub struct StrokePath {
    pub points: Vec<StrokePoint>,
    pub bounds: Box2d,
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
    pub framework: &'editor mut Framework,
    pub editor: &'stroke mut ImageEditor,
    pub renderer: &'stroke mut Renderer,
}

pub trait BrushEngine {
    fn begin_stroking(&mut self, _context: &mut EditorContext) -> Option<Box<dyn EditorCommand>> {
        None
    }
    fn stroke(
        &mut self,
        _path: StrokePath,
        _context: StrokeContext,
    ) -> Option<Box<dyn EditorCommand>> {
        None
    }
    fn end_stroking(&mut self, _context: &mut EditorContext) -> Option<Box<dyn EditorCommand>> {
        None
    }
}
impl StrokePath {
    pub(crate) fn linear_start_to_end(start: StrokePoint, end: StrokePoint, step: f32) -> Self {
        let bounds = Box2d {
            center: start.position,
            extents: vec2(start.size, start.size),
        }
        .union(&Box2d {
            center: end.position,
            extents: vec2(end.size, end.size),
        });
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
        StrokePath { points, bounds }
    }

    fn bounds(&self) -> Box2d {
        self.bounds
    }
}
