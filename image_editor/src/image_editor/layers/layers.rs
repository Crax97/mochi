use cgmath::{point3, ElementWise, Point2, Point3, Vector2};
use framework::framework::TextureId;
use framework::renderer::renderer::Renderer;
use framework::scene::Transform2d;
use framework::Framework;
use uuid::Uuid;

use crate::blend_settings::BlendMode;

use super::BitmapLayer;

#[derive(Clone, PartialEq)]
pub struct LayerSettings {
    pub name: String,
    pub is_enabled: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub struct ShaderLayerSettings {
    pub opacity: f32,
}

pub struct Layer {
    uuid: Uuid,
    needs_settings_update: bool,
    needs_bitmap_update: bool,
    pub settings: LayerSettings,
    pub position: Point2<f32>,
    pub scale: Vector2<f32>,
    pub rotation_radians: f32,

    pub layer_type: LayerType,
    pub bitmap: BitmapLayer,
}

pub struct LayerCreationInfo {
    pub name: String,
    pub position: Point2<f32>,
    pub scale: Vector2<f32>,
    pub rotation_radians: f32,
}

pub enum LayerType {
    Bitmap,
}

impl Layer {
    pub fn new_bitmap(bitmap: BitmapLayer, creation_info: LayerCreationInfo) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            bitmap,
            settings: LayerSettings {
                name: creation_info.name,
                is_enabled: true,
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
            },

            needs_settings_update: true,
            needs_bitmap_update: true,

            layer_type: LayerType::Bitmap,
            position: creation_info.position,
            scale: creation_info.scale,
            rotation_radians: creation_info.rotation_radians,
        }
    }

    pub fn needs_settings_update(&mut self) -> bool {
        let ret = self.needs_settings_update;
        self.needs_settings_update = false;
        ret
    }

    pub fn needs_bitmap_update(&mut self) -> bool {
        let ret = self.needs_bitmap_update;
        self.needs_bitmap_update = false;
        ret
    }

    pub fn mark_dirty(&mut self) {
        self.needs_bitmap_update = true;
    }

    pub fn translate(&mut self, delta: Vector2<f32>) {
        self.position += delta;
        self.mark_dirty();
    }

    pub fn replace_texture(&mut self, new_texture: TextureId) {
        self.bitmap.replace_texture(new_texture);
        self.mark_dirty();
    }

    pub fn settings(&self) -> &LayerSettings {
        &self.settings
    }

    pub fn set_settings(&mut self, new_settings: LayerSettings) {
        self.settings = new_settings;
        self.needs_settings_update = true;
        self.mark_dirty();
    }

    pub fn transform(&self) -> Transform2d {
        Transform2d {
            position: Point3 {
                x: self.position.x,
                y: self.position.y,
                z: 0.0,
            },
            scale: Vector2 {
                x: self.scale.x,
                y: self.scale.y,
            },
            rotation_radians: cgmath::Rad(self.rotation_radians),
        }
    }

    pub fn pixel_transform(&self) -> Transform2d {
        Transform2d {
            position: point3(self.position.x, self.position.y, 0.0),
            scale: self.bitmap.size().mul_element_wise(self.scale * 0.5),
            rotation_radians: cgmath::Rad(self.rotation_radians),
        }
    }

    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }
}
