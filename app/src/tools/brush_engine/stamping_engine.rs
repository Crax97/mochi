use std::cell::RefCell;
use std::rc::Rc;

use cgmath::vec2;
use framework::AssetsLibrary;
use framework::{Framework, MeshInstance2D, TypedBuffer};
use image::{ImageBuffer, Rgba};
use wgpu::{RenderPassColorAttachment, RenderPassDescriptor};

use crate::{StrokeContext, StrokePath};

use super::BrushEngine;

pub struct Stamp {}

impl Stamp {
    pub fn new(brush_texture: ImageBuffer<Rgba<u8>, Vec<u8>>) -> Self {
        Self {}
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
        match context.layer.layer_type {
            image_editor::layers::LayerType::Bitmap(ref bitmap_layer) => {
                // 1. Update buffer
                let instances: Vec<MeshInstance2D> = path
                    .points
                    .iter()
                    .map(|pt| MeshInstance2D::new(pt.position, vec2(pt.size, pt.size), 0.0))
                    .collect();
                // 2. Do draw
            }
        }
    }
}
