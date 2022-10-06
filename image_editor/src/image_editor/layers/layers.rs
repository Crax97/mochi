use cgmath::{Point2, Point3, Vector2, Vector3};
use framework::{Framework, MeshInstance2D, TypedBuffer, TypedBufferConfiguration};
use renderer::render_pass::texture2d_draw_pass::Texture2dDrawPass;
use scene::{Camera2d, Transform2d};
use wgpu::TextureView;

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
    pub settings: LayerSettings,
    pub layer_type: LayerType,
    pub position: Point2<f32>,
    pub scale: Vector2<f32>,
    pub rotation_radians: f32,
    pub instance_buffer: TypedBuffer<'framework>,
}

pub struct LayerCreationInfo {
    pub name: String,
    pub position: Point2<f32>,
    pub scale: Vector2<f32>,
    pub rotation_radians: f32,
}

pub enum LayerType {
    Bitmap(bitmap_layer::BitmapLayer),
}

impl<'framework> Layer<'framework> {
    pub fn new_bitmap(
        bitmap_layer: BitmapLayer,
        creation_info: LayerCreationInfo,
        framework: &'framework Framework,
    ) -> Self {
        let instance_buffer = framework.allocate_typed_buffer(TypedBufferConfiguration {
            initial_setup: framework::typed_buffer::BufferInitialSetup::Data(
                &Vec::<MeshInstance2D>::new(),
            ),
            buffer_type: framework::BufferType::Vertex,
            allow_write: true,
            allow_read: false,
        });

        Self {
            settings: LayerSettings {
                name: creation_info.name,
                is_enabled: true,
                opacity: 1.0,
            },
            layer_type: LayerType::Bitmap(bitmap_layer),
            position: creation_info.position,
            scale: creation_info.scale,
            rotation_radians: creation_info.rotation_radians,
            instance_buffer,
        }
    }
    pub(crate) fn update(&mut self) {
        match &self.layer_type {
            LayerType::Bitmap(bitmap_layer) => {
                let real_scale = Vector2 {
                    x: self.scale.x * bitmap_layer.size().x * 0.5,
                    y: self.scale.y * bitmap_layer.size().y * 0.5,
                };
                self.instance_buffer.write_sync(&vec![MeshInstance2D::new(
                    self.position.clone(),
                    real_scale,
                    self.rotation_radians,
                    true,
                    self.settings.opacity,
                )]);
            }
        }
    }
    pub(crate) fn draw<'library, 'pass, 'l>(
        &'l self,
        framework: &'framework Framework,
        pass: &mut Texture2dDrawPass<'framework>,
        target: &TextureView,
    ) where
        'framework: 'pass,
        'l: 'pass,
    {
        if !self.settings.is_enabled {
            return;
        }
        match &self.layer_type {
            LayerType::Bitmap(ref bm) => {
                let real_scale = Vector2 {
                    x: self.scale.x * bm.size().x * 0.5,
                    y: self.scale.y * bm.size().y * 0.5,
                };
                pass.begin(&Camera2d::new(
                    -0.1,
                    1000.0,
                    [-real_scale.x, real_scale.x, real_scale.y, -real_scale.y],
                ));
                pass.draw_texture(
                    bm.texture(),
                    MeshInstance2D::new(
                        self.position,
                        real_scale,
                        self.rotation_radians,
                        true,
                        self.settings.opacity,
                    ),
                );
                pass.execute(framework, target, false);
            }
        }
    }

    pub fn settings(&self) -> LayerSettings {
        self.settings.clone()
    }

    pub fn set_settings(&mut self, new_settings: LayerSettings) {
        self.settings = new_settings;
        self.instance_buffer.write_sync(&vec![MeshInstance2D::new(
            self.position,
            self.scale,
            self.rotation_radians,
            true,
            self.settings.opacity,
        )])
    }

    pub fn transform(&self) -> Transform2d {
        Transform2d {
            position: Point3 {
                x: self.position.x,
                y: self.position.y,
                z: 0.0,
            },
            scale: Vector3 {
                x: self.scale.x,
                y: self.scale.y,
                z: 1.0,
            },
            rotation_radians: cgmath::Rad(self.rotation_radians),
        }
    }
}
