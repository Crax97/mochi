use cgmath::{vec4, Point2, Vector2, Vector4};
use wgpu::{VertexAttribute, VertexBufferLayout};

use crate::Mesh;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MeshInstance2D {
    position_and_scale: Vector4<f32>,
    rotation: f32,
}

impl MeshInstance2D {
    pub fn new(position: Point2<f32>, scale: Vector2<f32>, rotation_rads: f32) -> Self {
        Self {
            position_and_scale: vec4(position.x, position.y, scale.x, scale.y),
            rotation: rotation_rads,
        }
    }
}

impl<'a> MeshInstance2D {
    pub fn layout() -> VertexBufferLayout<'a> {
        const LAYOUT: &'static [VertexAttribute] =
            &wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32];
        VertexBufferLayout {
            array_stride: std::mem::size_of::<MeshInstance2D>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: LAYOUT,
        }
    }
}

unsafe impl bytemuck::Pod for MeshInstance2D {}
unsafe impl bytemuck::Zeroable for MeshInstance2D {}
