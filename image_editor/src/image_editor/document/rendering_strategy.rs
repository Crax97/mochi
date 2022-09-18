use image::{ImageBuffer, Rgba};

use crate::layers::Layer;

pub trait RenderingStrategy {
    fn begin(&mut self);
    fn render_layer(&mut self, layer: &Layer);
    fn end(&mut self) {}
    fn get_result(&self) -> &ImageBuffer<Rgba<u8>, Vec<u8>>;
}
