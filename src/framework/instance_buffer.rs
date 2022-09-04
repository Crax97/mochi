use std::cell::RefCell;

use wgpu::{util::DeviceExt, RenderPass};

use super::Framework;

pub struct InstanceBuffer {
    buffer: wgpu::Buffer,
}

pub struct InstanceBufferConfiguration<T> {
    pub initial_data: Vec<T>,
    pub allow_write: bool,
}

impl InstanceBuffer {
    pub fn new<T: bytemuck::Pod + bytemuck::Zeroable>(
        framework: &Framework,
        initial_configuration: InstanceBufferConfiguration<T>,
    ) -> Self {
        use std::mem;
        let usage = wgpu::BufferUsages::VERTEX
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
        InstanceBuffer { buffer }
    }

    pub fn bind<'a>(&'a self, index: u32, render_pass: &RefCell<RenderPass<'a>>) {
        render_pass
            .borrow_mut()
            .set_vertex_buffer(index, self.buffer.slice(..));
    }
}
