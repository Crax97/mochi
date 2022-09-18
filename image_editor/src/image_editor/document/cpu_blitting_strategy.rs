use image::{EncodableLayout, ImageBuffer, Rgba};
use pix::{
    el::{PixRgba, Pixel},
    rgb::Rgba8p,
    Raster,
};

use crate::layers::LayerType;

use super::RenderingStrategy;

pub struct CpuRenderingStrategy {
    result: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

impl CpuRenderingStrategy {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            result: ImageBuffer::<_, _>::new(width, height),
        }
    }
}

impl RenderingStrategy for CpuRenderingStrategy {
    fn begin(&mut self) {
        for p in self.result.iter_mut() {
            *p = 255;
        }
    }

    fn render_layer(&mut self, layer: &crate::layers::Layer) {
        if !layer.settings.is_enabled {
            return;
        }

        match &layer.layer_type {
            LayerType::Bitmap(layer_bitmap) => {
                let raster_src = Raster::<PixRgba<Rgba8p>>::with_u8_buffer(
                    layer_bitmap.width(),
                    layer_bitmap.height(),
                    layer_bitmap.as_raw().as_bytes(),
                );
                let mut raster_dest = Raster::<PixRgba<Rgba8p>>::with_u8_buffer(
                    self.result.width(),
                    self.result.height(),
                    self.result.as_raw().as_bytes(),
                );

                PixRgba::<Rgba8p>::composite_slice(
                    raster_dest.pixels_mut(),
                    raster_src.pixels(),
                    pix::ops::SrcOver,
                );
                self.result.copy_from_slice(raster_dest.as_u8_slice());
            }
        }
    }

    fn get_result(&self) -> &ImageBuffer<Rgba<u8>, Vec<u8>> {
        &self.result
    }
}
