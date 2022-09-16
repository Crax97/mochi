use cgmath::{Point2, Vector2};
use framework::{Framework, MeshInstance2D, TypedBuffer, TypedBufferConfiguration};
use image::{ImageBuffer, Rgba};
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry};

#[derive(Clone, PartialEq)]
pub struct LayerSettings {
    pub name: String,
    pub is_enabled: bool,
    pub opacity: f32,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, bytemuck::Zeroable, bytemuck::Pod)]
pub struct ShaderLayerSettings {
    pub opacity: f32,
}

pub struct Layer {
    pub layer_type: LayerType,
    pub settings: LayerSettings,
    pub position: Point2<f32>,
    pub scale: Vector2<f32>,
    pub rotation_radians: f32,
}

pub struct LayerCreationInfo {
    pub name: String,
    pub initial_color: Rgba<u8>,
    pub width: u32,
    pub height: u32,
    pub position: Point2<f32>,
    pub scale: Vector2<f32>,
    pub rotation_radians: f32,
}

pub enum LayerType {
    Bitmap(image::ImageBuffer<Rgba<u8>, Vec<u8>>),
}

pub(crate) struct LayerDrawContext {}

impl Layer {
    pub fn new_bitmap(creation_info: LayerCreationInfo) -> Self {
        let mut buffer = ImageBuffer::new(creation_info.width, creation_info.height);
        for p in buffer.pixels_mut() {
            *p = creation_info.initial_color.clone();
        }
        Self {
            settings: LayerSettings {
                name: creation_info.name,
                is_enabled: true,
                opacity: 1.0,
            },
            layer_type: LayerType::Bitmap(buffer),
            position: creation_info.position,
            scale: creation_info.scale,
            rotation_radians: creation_info.rotation_radians,
        }
    }
    pub(crate) fn update(&mut self) {}
    pub(crate) fn draw(&self, _: &mut LayerDrawContext) {
        if !self.settings.is_enabled {
            return;
        }
        match &self.layer_type {
            LayerType::Bitmap(_) => {}
        }
    }

    pub fn settings(&self) -> LayerSettings {
        self.settings.clone()
    }

    pub fn set_settings(&mut self, new_settings: LayerSettings) {
        self.settings = new_settings;
    }
}
