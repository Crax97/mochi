use std::cell::RefCell;

use as_slice::AsSlice;
use wgpu::{util::DeviceExt, BufferUsages, RenderPass};

use super::framework::Framework;

#[derive(Copy, Clone, Debug)]
pub enum BufferType {
    // A buffer meant to be used as an input for the Vertex Shader
    Vertex,
    // A buffer whose content doesn't change during a draw invocation
    Uniform,
    // A buffer whose contents can be dynamic
    Storage,
}

struct BufferInfo {
    pub buffer: wgpu::Buffer,
    pub num_items: usize,
}

pub struct InnerBufferConfiguration {
    pub buffer_type: BufferType,
    pub allow_write: bool,
    pub allow_read: bool,
}
pub struct TypedBuffer<'framework> {
    buffer: BufferInfo,
    configuration: InnerBufferConfiguration,
    owner_framework: &'framework Framework,
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

fn recreate_buffer<T: AsSlice>(
    data: &T,
    config: &InnerBufferConfiguration,
    framework: &'_ Framework,
) -> BufferInfo
where
    T::Element: bytemuck::Pod + bytemuck::Zeroable,
{
    use std::mem;
    let buffer_usage: BufferUsages = config.buffer_type.into();
    let usage: BufferUsages = buffer_usage
        | if config.allow_write {
            wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_DST
        } else {
            wgpu::BufferUsages::empty()
        }
        | if config.allow_write {
            wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_SRC
        } else {
            wgpu::BufferUsages::empty()
        };
    let buffer = if data.as_slice().len() > 0 {
        framework
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &bytemuck::cast_slice(data.as_slice()),
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
    BufferInfo {
        buffer,
        num_items: data.as_slice().len(),
    }
}

impl<'framework> TypedBuffer<'framework> {
    pub fn new<T: bytemuck::Pod + bytemuck::Zeroable>(
        framework: &'framework Framework,
        initial_configuration: TypedBufferConfiguration<T>,
    ) -> Self {
        let configuration = InnerBufferConfiguration {
            allow_read: initial_configuration.allow_read,
            allow_write: initial_configuration.allow_write,
            buffer_type: initial_configuration.buffer_type,
        };
        let buffer = recreate_buffer(
            &initial_configuration.initial_data.as_slice(),
            &configuration,
            framework,
        );
        TypedBuffer {
            buffer: buffer,
            configuration,
            owner_framework: framework,
        }
    }

    pub fn write_sync<T: AsSlice>(&mut self, data: &T)
    where
        T::Element: bytemuck::Pod + bytemuck::Zeroable,
    {
        let queue = &self.owner_framework.queue;
        let length = data.as_slice().len();
        let current_items = self.buffer.num_items;
        if length > current_items {
            self.buffer = recreate_buffer(data, &self.configuration, self.owner_framework);
        }
        let buffer = &self.buffer.buffer;
        queue.write_buffer(&buffer, 0, &bytemuck::cast_slice(&data.as_slice()));
    }

    pub fn bind<'a>(&'a self, index: u32, render_pass: &mut RenderPass<'a>) {
        match self.configuration.buffer_type {
            BufferType::Vertex => {
                let buffer = &self.buffer.buffer;

                render_pass.set_vertex_buffer(index, buffer.slice(..));
            }
            BufferType::Uniform => {
                panic!("Uniform buffers should be set by using the associated bind group!")
            }
            BufferType::Storage => todo!(),
        };
    }

    pub fn binding_resource(&self) -> wgpu::BufferBinding {
        let buffer = &self.buffer.buffer;
        buffer.as_entire_buffer_binding()
    }
}
