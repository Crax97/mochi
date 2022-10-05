use cgmath::{vec4, Point2, Vector2, Vector4};
use wgpu::{VertexAttribute, VertexBufferLayout};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MeshInstance2D {
    position_and_scale: Vector4<f32>,
    rotation_flip_opacity: Vector4<f32>,
}

impl MeshInstance2D {
    pub fn new(
        position: Point2<f32>,
        scale: Vector2<f32>,
        rotation_rads: f32,
        flip_y: bool,
        opacity: f32,
    ) -> Self {
        Self {
            position_and_scale: vec4(position.x, position.y, scale.x, scale.y),
            rotation_flip_opacity: Vector4 {
                x: rotation_rads,
                y: if flip_y { 1.0 } else { 0.0 },
                z: opacity,
                w: 1.0,
            },
        }
    }
}

impl<'a> MeshInstance2D {
    pub fn layout() -> VertexBufferLayout<'a> {
        const LAYOUT: &'static [VertexAttribute] =
            &wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x4];
        VertexBufferLayout {
            array_stride: std::mem::size_of::<MeshInstance2D>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: LAYOUT,
        }
    }
}

unsafe impl bytemuck::Pod for MeshInstance2D {}
unsafe impl bytemuck::Zeroable for MeshInstance2D {}
