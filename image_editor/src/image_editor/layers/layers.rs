use cgmath::{Point2, Point3, Vector2};
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

pub struct Layer<'framework> {
    framework: &'framework Framework,

    uuid: Uuid,
    needs_settings_update: bool,
    needs_bitmap_update: bool,
    settings: LayerSettings,
    position: Point2<f32>,
    scale: Vector2<f32>,
    rotation_radians: f32,

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

impl<'framework> Layer<'framework> {
    pub fn new_bitmap(
        bitmap: BitmapLayer,
        creation_info: LayerCreationInfo,
        framework: &'framework Framework,
    ) -> Self {
        Self {
            framework,
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

    pub(crate) fn lay_on_canvas(&self, renderer: &mut Renderer, canvas: &BitmapLayer) {
        renderer.begin(&canvas.camera(), None);
        match self.layer_type {
            LayerType::Bitmap => {
                self.bitmap.draw(
                    renderer,
                    self.position,
                    self.scale,
                    self.rotation_radians,
                    self.settings.opacity,
                    canvas.texture(),
                );
            }
        }
        renderer.end_on_texture(canvas.texture());
    }

    pub fn settings(&self) -> &LayerSettings {
        &self.settings
    }

    pub fn set_settings(&mut self, new_settings: LayerSettings) {
        self.settings = new_settings;
        self.mark_dirty();
        self.needs_settings_update = true;
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

    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }
}
