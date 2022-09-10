pub mod stamping_engine;

use std::{cell::RefCell, rc::Rc};

use cgmath::{InnerSpace, Point2};
use framework::{asset_library::AssetsLibrary, Debug};
use wgpu::CommandEncoder;

use crate::{layers::Layer, ImageEditor};

#[derive(Debug)]
pub struct StrokePath {
    pub points: Vec<Point2<f32>>,
}

impl std::fmt::Display for StrokePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("StrokePath {\n")?;
        f.write_str("\tpoints: [\n")?;
        for pt in self.points.iter() {
            f.write_str(format!("\t\tPoint2<f32> {{ x: {}, y: {} }},\n", pt.x, pt.y).as_str())?;
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
    pub assets: &'editor AssetsLibrary,
    pub debug: Rc<RefCell<Debug>>,
}

pub trait BrushEngine {
    fn stroke(&mut self, path: StrokePath, context: StrokeContext);
}
impl StrokePath {
    pub(crate) fn linear_start_to_end(start: Point2<f32>, end: Point2<f32>, step: f32) -> Self {
        let direction = end - start;
        let distance = direction.magnitude();
        let direction = direction.normalize();
        let num_points = (distance / step) as usize;
        let points = (0..num_points)
            .into_iter()
            .map(|pt| start + ((pt as f32 * step) * direction))
            .chain(std::iter::once(end))
            .collect();
        StrokePath { points }
    }
}
