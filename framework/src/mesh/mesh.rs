use wgpu::{util::DeviceExt, Buffer, RenderPass, VertexAttribute, VertexBufferLayout};

use super::types::*;
use crate::framework::Framework;

const VERTEX_BUFFER_POSITION: u32 = 0;

pub struct MeshConstructionDetails {
    pub vertices: Vertices,
    pub indices: Indices,
    pub allow_editing: bool,
}

unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

pub struct Mesh {
    vertices_vertex_buffer: Buffer,
    index_buffer: Buffer,
    construction_details: MeshConstructionDetails,
}

impl<'a> Mesh {
    pub fn layout() -> VertexBufferLayout<'a> {
        const LAYOUT: &'static [VertexAttribute] =
            &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: LAYOUT,
        }
    }
}

impl Mesh {
    pub fn new(framework: &Framework, construction_details: MeshConstructionDetails) -> Self {
        let vertices_vertex_buffer =
            framework
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: &bytemuck::cast_slice(&construction_details.vertices.0),
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::MAP_READ
                        | if construction_details.allow_editing {
                            wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_DST
                        } else {
                            wgpu::BufferUsages::empty()
                        },
                });
        let index_buffer = framework
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &bytemuck::cast_slice(&construction_details.indices.0),
                usage: wgpu::BufferUsages::INDEX
                    | wgpu::BufferUsages::MAP_READ
                    | if construction_details.allow_editing {
                        wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_DST
                    } else {
                        wgpu::BufferUsages::empty()
                    },
            });
        Mesh {
            vertices_vertex_buffer,
            index_buffer,
            construction_details,
        }
    }

    pub fn draw<'a, 'pass>(&'a self, render_pass: &mut RenderPass<'pass>, instance_count: u32)
    where
        'a: 'pass,
    {
        render_pass.set_index_buffer(self.index_buffer.slice(..), INDEX_FORMAT);
        render_pass.set_vertex_buffer(
            VERTEX_BUFFER_POSITION,
            self.vertices_vertex_buffer.slice(..),
        );
        render_pass.draw_indexed(
            0..self.construction_details.indices.0.len() as u32,
            0,
            0..instance_count,
        )
    }
}
