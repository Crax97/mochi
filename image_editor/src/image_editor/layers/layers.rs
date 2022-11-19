use cgmath::{point3, vec2, ElementWise, Point2, Point3, Vector2};
use framework::framework::TextureId;
use framework::scene::Transform2d;
use framework::{Framework, RgbaTexture2D, Texture, TextureConfiguration, TextureUsage};
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
    pub settings: LayerSettings,
    pub position: Point2<f32>,
    pub scale: Vector2<f32>,
    pub rotation_radians: f32,

    pub layer_type: LayerType,
    needs_settings_update: bool,
    needs_bitmap_update: bool,
}

pub struct LayerCreationInfo {
    pub name: String,
    pub position: Point2<f32>,
    pub scale: Vector2<f32>,
    pub rotation_radians: f32,
}

pub enum LayerType {
    Image {
        texture: TextureId,
        dimensions: Vector2<u32>,
    },
    Group(Vec<Uuid>),
}

impl Layer {
    pub fn new_image(
        image: RgbaTexture2D,
        creation_info: LayerCreationInfo,
        framework: &mut Framework,
    ) -> Self {
        let (w, h) = (image.width(), image.height());
        let texture = framework.allocate_texture2d(
            image,
            TextureConfiguration {
                label: Some(format!("Layer \"{}\" texture", creation_info.name).as_str()),
                usage: TextureUsage::RWRT,
                mip_count: None,
            },
        );
        Self {
            uuid: Uuid::new_v4(),

            layer_type: LayerType::Image {
                texture,
                dimensions: vec2(w, h),
            },
            settings: LayerSettings {
                name: creation_info.name,
                is_enabled: true,
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
            },

            needs_settings_update: true,
            needs_bitmap_update: true,
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
        match &mut self.layer_type {
            LayerType::Image { texture, .. } => *texture = new_texture,
            LayerType::Group(_) => unreachable!(),
        };
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
            scale: self
                .size()
                .cast::<f32>()
                .unwrap()
                .mul_element_wise(self.scale),
            rotation_radians: cgmath::Rad(self.rotation_radians),
        }
    }

    pub fn size(&self) -> Vector2<u32> {
        match self.layer_type {
            LayerType::Image { dimensions, .. } => dimensions.clone(),
            LayerType::Group(_) => unreachable!(),
        }
    }

    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }
}
