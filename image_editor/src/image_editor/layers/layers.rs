use cgmath::{Point2, Point3, Vector2};
use framework::framework::{ShaderId, TextureId};
use framework::renderer::renderer::Renderer;
use framework::scene::Transform2d;
use framework::{framework::BufferId, BufferConfiguration, Framework, MeshInstance2D};

use super::{bitmap_layer, BitmapLayer};

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

pub struct Layer<'framework> {
    framework: &'framework Framework,
    pub bitmap: BitmapLayer,
    pub settings: LayerSettings,
    pub layer_type: LayerType,
    pub position: Point2<f32>,
    pub scale: Vector2<f32>,
    pub rotation_radians: f32,
    pub instance_buffer_id: BufferId,
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
        let instance_buffer_id = framework.allocate_typed_buffer(BufferConfiguration {
            initial_setup: framework::buffer::BufferInitialSetup::Data(
                &Vec::<MeshInstance2D>::new(),
            ),
            buffer_type: framework::BufferType::Vertex,
            allow_write: true,
            allow_read: false,
        });

        Self {
            framework,
            bitmap,
            settings: LayerSettings {
                name: creation_info.name,
                is_enabled: true,
                opacity: 1.0,
            },
            layer_type: LayerType::Bitmap,
            position: creation_info.position,
            scale: creation_info.scale,
            rotation_radians: creation_info.rotation_radians,
            instance_buffer_id,
        }
    }

    fn wgpu_color(&self) -> wgpu::Color {
        wgpu::Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: self.settings.opacity as f64,
        }
    }

    pub(crate) fn update(&mut self, framework: &Framework) {}

    pub(crate) fn draw<'library, 'pass, 'l>(
        &'l self,
        renderer: &mut Renderer,
        bottom_layer: &TextureId,
        target: &TextureId,
    ) where
        'framework: 'pass,
        'l: 'pass,
    {
        if !self.settings.is_enabled {
            return;
        }
        match &self.layer_type {
            LayerType::Bitmap => {
                self.bitmap.draw(
                    renderer,
                    self.position,
                    self.scale,
                    self.rotation_radians,
                    self.settings.opacity,
                    target,
                );
            }
        }
    }

    pub fn settings(&self) -> LayerSettings {
        self.settings.clone()
    }

    pub fn set_settings(&mut self, new_settings: LayerSettings) {
        self.settings = new_settings;

        self.framework.buffer_write_sync(
            &self.instance_buffer_id,
            vec![MeshInstance2D::new(
                self.position,
                self.scale,
                self.rotation_radians,
                true,
                wgpu::Color::WHITE,
            )],
        );
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
}
