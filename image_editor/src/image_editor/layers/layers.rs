use std::borrow::Borrow;
use std::cell::Ref;

use cgmath::{Point2, Vector2};
use framework::render_pass::{PassBindble, RenderPass};
use framework::{Framework, MeshInstance2D, TypedBuffer, TypedBufferConfiguration};
use scene::Camera2d;
use wgpu::BindGroup;

use framework::{asset_library::AssetsLibrary, mesh_names};

use super::layer_draw_pass::LayerDrawPass;
use super::{bitmap_layer, BitmapLayer};

pub struct Layer<'framework> {
    pub name: String,
    pub layer_type: LayerType<'framework>,
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

pub enum LayerType<'framework> {
    Bitmap(bitmap_layer::BitmapLayer<'framework>),
}

pub(crate) struct LayerDrawContext<'context, 'library, 'pass> {
    pub render_pass: wgpu::RenderPass<'pass>,
    pub draw_pass: &'context LayerDrawPass,
    pub assets: Ref<'library, AssetsLibrary>,
}

impl<'framework> Layer<'framework> {
    pub fn new_bitmap(
        bitmap_layer: BitmapLayer<'framework>,
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
            name: creation_info.name,
            layer_type: LayerType::Bitmap(bitmap_layer),
            position: creation_info.position,
            scale: creation_info.scale,
            instance_buffer,
            rotation_radians: creation_info.rotation_radians,
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
                )]);
            }
        }
    }
    pub(crate) fn draw<'context, 'library, 'pass, 'l>(
        &'l self,
        mut draw_context: LayerDrawContext<'context, 'library, 'pass>,
    ) where
        'framework: 'pass,
        'l: 'pass,
        'context: 'pass,
    {
        match &self.layer_type {
            LayerType::Bitmap(bm) => {
                draw_context.draw_pass.execute_with_renderpass(
                    draw_context.render_pass,
                    &[
                        (1, &self.instance_buffer),
                        (0, self.bind_group()),
                        (1, self.camera_bind_group()),
                    ],
                );
            }
        }
    }
    pub(crate) fn size(&self) -> Vector2<f32> {
        match self.layer_type {
            LayerType::Bitmap(ref bm) => bm.size(),
        }
    }

    pub(crate) fn bind_group(&self) -> &BindGroup {
        match self.layer_type {
            LayerType::Bitmap(ref bm) => bm.bind_group(),
        }
    }

    fn camera_bind_group(&self) -> &BindGroup {
        match self.layer_type {
            LayerType::Bitmap(ref bm) => bm.camera_bind_group(),
        }
    }
}
