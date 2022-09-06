mod bitmap_layer;

use std::{cell::RefCell, rc::Rc};

pub use bitmap_layer::*;
use cgmath::{Point2, Vector2};
use wgpu::RenderPass;

use crate::framework::{Framework, MeshInstance2D, TypedBuffer, TypedBufferConfiguration};

use super::Assets;

pub struct Layer {
    pub layer_type: LayerType,
    pub position: Point2<f32>,
    pub scale: Vector2<f32>,
    pub rotation_radians: f32,

    pub instance_buffer: TypedBuffer,
}
pub enum LayerType {
    Bitmap(bitmap_layer::BitmapLayer),
}

pub(crate) struct LayerDrawContext<'a, 'b> {
    pub render_pass: &'b mut RenderPass<'a>,
    pub assets: &'a Assets,
}

impl Layer {
    pub fn update(&mut self, framework: &Framework) {
        self.instance_buffer = TypedBuffer::new(
            &framework,
            TypedBufferConfiguration {
                initial_data: vec![MeshInstance2D {
                    position: self.position.clone(),
                    scale: self.scale.clone(),
                    rotation: self.rotation_radians,
                }],
                buffer_type: crate::framework::BufferType::Vertex,
                allow_write: false,
            },
        );
    }

    pub(crate) fn draw<'a, 'b>(&'a self, draw_context: &mut LayerDrawContext<'a, 'b>) {
        match &self.layer_type {
            LayerType::Bitmap(bitmap_layer) => {
                self.instance_buffer.bind(1, draw_context.render_pass);
                draw_context
                    .render_pass
                    .set_bind_group(0, bitmap_layer.binding_group(), &[]);
                draw_context
                    .assets
                    .quad_mesh
                    .draw(draw_context.render_pass, 1);
            }
        }
    }
}
