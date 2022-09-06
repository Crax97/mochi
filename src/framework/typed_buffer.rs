use as_slice::AsSlice;
use wgpu::{util::DeviceExt, BufferUsages, RenderPass};

use super::Framework;

#[derive(Copy, Clone, Debug)]
pub enum BufferType {
    // A buffer meant to be used as an input for the Vertex Shader
    Vertex,
    // A buffer whose content doesn't change during a draw invocation
    Uniform,
    // A buffer whose contents can be dynamic
    Storage,
}

#[derive(Debug)]
pub struct TypedBuffer {
    buffer: wgpu::Buffer,
    buffer_type: BufferType,
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
    pub allow_read: bool,
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
                wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_DST
            } else {
                wgpu::BufferUsages::empty()
            }
            | if initial_configuration.allow_write {
                wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_SRC
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
        TypedBuffer {
            buffer,
            buffer_type: initial_configuration.buffer_type,
        }
    }

    pub fn write_sync<T: AsSlice>(&self, data: &T, framework: &Framework)
    where
        T::Element: bytemuck::Pod + bytemuck::Zeroable,
    {
        let queue = &framework.queue;
        queue.write_buffer(&self.buffer, 0, &bytemuck::cast_slice(&data.as_slice()));
    }

    pub fn bind<'a>(&'a self, index: u32, render_pass: &mut RenderPass<'a>) {
        match self.buffer_type {
            BufferType::Vertex => render_pass.set_vertex_buffer(index, self.buffer.slice(..)),
            BufferType::Uniform => {
                panic!("Uniform buffers should be set by using the associated bind group!")
            }
            BufferType::Storage => todo!(),
        };
    }

    pub(crate) fn binding_resource(&self) -> wgpu::BufferBinding {
        self.buffer.as_entire_buffer_binding()
    }
}
