use cgmath::{Point2, Vector2};
use wgpu::{VertexAttribute, VertexBufferLayout};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MeshInstance2D {
    pub position: Point2<f32>,
    pub scale: Vector2<f32>,
    pub rotation: f32,
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
