use wgpu::{util::DeviceExt, BufferUsages, RenderPass};

use super::Framework;

pub struct TypedBuffer {
    buffer: wgpu::Buffer,
}

pub enum BufferType {
    // A buffer meant to be used as an input for the Vertex Shader
    Vertex,
    // A buffer whose content doesn't change during a draw invocation
    Uniform,
    // A buffer whose contents can be dynamic
    Storage,
}

impl From<BufferType> for BufferUsages {
    fn from(buffer_type: BufferType) -> Self {
        match buffer_type {
            BufferType::Vertex => wgpu::BufferUsages::VERTEX,
            BufferType::Uniform => wgpu::BufferUsages::UNIFORM,
            BufferType::Storage => wgpu::BufferUsages::STORAGE,
        }
    }
}

pub struct TypedBufferConfiguration<T> {
    pub initial_data: Vec<T>,
    pub buffer_type: BufferType,
    pub allow_write: bool,
}

impl TypedBuffer {
    pub fn new<T: bytemuck::Pod + bytemuck::Zeroable>(
        framework: &Framework,
        initial_configuration: TypedBufferConfiguration<T>,
    ) -> Self {
        use std::mem;
        let buffer_usage: BufferUsages = initial_configuration.buffer_type.into();
        let usage: BufferUsages = buffer_usage
            | if initial_configuration.allow_write {
                wgpu::BufferUsages::MAP_WRITE
            } else {
                wgpu::BufferUsages::empty()
            };
        let buffer = if initial_configuration.initial_data.len() > 0 {
            framework
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: &bytemuck::cast_slice(&initial_configuration.initial_data),
                    usage,
                })
        } else {
            framework.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: mem::size_of::<T>() as u64,
                usage,
                mapped_at_creation: false,
            })
        };
        TypedBuffer { buffer }
    }

    pub fn bind<'a>(&'a self, index: u32, render_pass: &mut RenderPass<'a>) {
        render_pass.set_vertex_buffer(index, self.buffer.slice(..));
    }
}
