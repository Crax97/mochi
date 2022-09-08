pub mod stamping_engine;

use cgmath::{InnerSpace, MetricSpace, Point2};
use wgpu::CommandEncoder;

use crate::{layers::Layer, AssetsLibrary, ImageEditor};

#[derive(Debug)]
pub struct StrokePath {
    pub points: Vec<Point2<f32>>,
}

pub struct StrokeContext<'editor, 'stroke> {
    pub layer: &'editor Layer<'editor>,
    pub editor: &'stroke ImageEditor<'editor>,
    pub command_encoder: &'stroke mut CommandEncoder,
    pub assets: &'editor AssetsLibrary,
}

pub trait BrushEngine {
    fn stroke(&mut self, path: StrokePath, context: StrokeContext);
}
impl StrokePath {
    pub(crate) fn linear_start_to_end(start: Point2<f32>, end: Point2<f32>, step: f32) -> Self {
        let direction = end - start;
        let distance = direction.magnitude() as usize;
        let direction = direction.normalize();
        let points = (0..distance)
            .step_by(step as usize)
            .into_iter()
            .map(|d| start + d as f32 * direction)
            .chain(std::iter::once(end))
            .collect();
        StrokePath { points }
    }
}
