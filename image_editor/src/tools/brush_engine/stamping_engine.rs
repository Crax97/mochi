use super::BrushEngine;

pub struct StrokingEngine {}

impl BrushEngine for StrokingEngine {
    fn stroke(&self, _layer: &crate::layers::Layer, _path: super::StrokePath) {}
}
