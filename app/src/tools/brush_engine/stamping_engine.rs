use std::cell::RefCell;
use std::rc::Rc;

use framework::AssetsLibrary;
use image::GenericImageView;
use image::{EncodableLayout, ImageBuffer, Rgba};
use pix::{Raster, Region};

use crate::{StrokeContext, StrokePath};

use super::BrushEngine;

pub struct Stamp {
    brush_texture: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

impl Stamp {
    pub fn new(brush_texture: ImageBuffer<Rgba<u8>, Vec<u8>>) -> Self {
        Self { brush_texture }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StampConfiguration {
    pub color_srgb: [u8; 3],
    pub opacity: u8,
    pub flow: f32,
    pub softness: f32,
    pub is_eraser: bool,
}

pub struct StrokingEngine {
    current_stamp: usize,
    stamps: Vec<Stamp>,
    configuration: StampConfiguration,
}

impl StrokingEngine {
    pub fn new(initial_stamp: Stamp, assets: Rc<RefCell<AssetsLibrary>>) -> Self {
        Self {
            stamps: vec![initial_stamp],
            current_stamp: 0,
            configuration: StampConfiguration {
                color_srgb: [0, 0, 0],
                opacity: 255,
                flow: 0.0,
                softness: 0.2,
                is_eraser: false,
            },
        }
    }

    pub fn create_stamp(&self, brush_texture: ImageBuffer<Rgba<u8>, Vec<u8>>) -> Stamp {
        Stamp::new(brush_texture)
    }

    pub fn settings(&self) -> StampConfiguration {
        self.configuration
    }

    pub fn set_new_settings(&mut self, settings: StampConfiguration) {
        self.configuration = settings;
    }

    fn current_stamp(&self) -> &Stamp {
        self.stamps
            .get(self.current_stamp)
            .expect("Could not find the given index in stamp array")
    }
}

impl BrushEngine for StrokingEngine {
    fn stroke(&mut self, path: StrokePath, context: StrokeContext) {
        let layer_mut = context.editor.mutate_document().current_layer_mut();
        match layer_mut.layer_type {
            image_editor::layers::LayerType::Bitmap(ref mut bitmap_layer) => {
                let raster_src = Raster::<pix::rgb::Rgba8>::with_u8_buffer(
                    self.current_stamp().brush_texture.width(),
                    self.current_stamp().brush_texture.height(),
                    self.current_stamp().brush_texture.as_raw().as_bytes(),
                );
                let mut raster_dest = Raster::<pix::rgb::Rgba8>::with_u8_buffer(
                    bitmap_layer.width(),
                    bitmap_layer.height(),
                    bitmap_layer.as_raw().as_bytes(),
                );
                raster_dest.copy_raster(
                    Region::new(
                        200,
                        200,
                        self.current_stamp().brush_texture.width(),
                        self.current_stamp().brush_texture.height(),
                    ),
                    &raster_src,
                    Region::new(
                        0,
                        0,
                        self.current_stamp().brush_texture.width(),
                        self.current_stamp().brush_texture.height(),
                    ),
                );
                bitmap_layer.put_pixel(0, 0, Rgba([128, 255, 128, 255]));
                //                bitmap_layer.copy_from_slice(raster_dest.as_u8_slice());
            }
        }
    }
}
