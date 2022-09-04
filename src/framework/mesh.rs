use cgmath::{Point2, Point3, Vector2, Vector3};
use wgpu::{util::DeviceExt, Buffer, RenderPass, VertexAttribute, VertexBufferLayout};

use super::Framework;

pub type Index = u16;
const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint16;
const VERTEX_BUFFER_POSITION: u32 = 0;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: Point3<f32>,
    pub tex_coords: Point2<f32>,
}
pub struct Indices(Vec<Index>);
pub struct Vertices(Vec<Vertex>);

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

    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>, instance_count: u32) {
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

impl<T: as_slice::AsSlice + IntoIterator> From<T> for Indices
where
    T::Item: Into<Index>,
{
    fn from(slice: T) -> Self {
        let index_vec: Vec<Index> = slice.into_iter().map(|i| i.into()).collect();
        Self(index_vec)
    }
}

impl<T: as_slice::AsSlice + IntoIterator> From<T> for Vertices
where
    T::Item: Into<Vertex>,
{
    fn from(slice: T) -> Self {
        let vertices_vec: Vec<Vertex> = slice.into_iter().map(|i| i.into()).collect();
        Self(vertices_vec)
    }
}
