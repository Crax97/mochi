use cgmath::{vec4, Point2, Vector2, Vector4};
use wgpu::{VertexAttribute, VertexBufferLayout};

use crate::shader::ShaderLayout;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MeshInstance2D {
    position_and_scale: Vector4<f32>,
    rotation_flip: Vector4<f32>,
    multiply_color: Vector4<f32>,
}

impl MeshInstance2D {
    pub fn new(
        position: Point2<f32>,
        scale: Vector2<f32>,
        rotation_rads: f32,
        flip_y: bool,
        multiply_color: wgpu::Color,
    ) -> Self {
        Self {
            position_and_scale: vec4(position.x, position.y, scale.x, scale.y),
            rotation_flip: Vector4 {
                x: rotation_rads,
                y: if flip_y { 1.0 } else { 0.0 },
                z: 1.0,
                w: 1.0,
            },
            multiply_color: Vector4 {
                x: multiply_color.r as f32,
                y: multiply_color.g as f32,
                z: multiply_color.b as f32,
                w: multiply_color.a as f32,
            },
        }
    }
}

impl ShaderLayout for MeshInstance2D {
    fn layout() -> VertexBufferLayout<'static> {
        const LAYOUT: &'static [VertexAttribute] =
            &wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x4, 4 => Float32x4];
        VertexBufferLayout {
            array_stride: std::mem::size_of::<MeshInstance2D>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: LAYOUT,
        }
    }
}

unsafe impl bytemuck::Pod for MeshInstance2D {}
unsafe impl bytemuck::Zeroable for MeshInstance2D {}
