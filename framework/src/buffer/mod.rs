use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, BufferSlice, BufferUsages};

use super::framework::Framework;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BufferType {
    // A buffer meant to be used as an input for the Vertex Shader
    Vertex,
    // A buffer whose content doesn't change during a draw invocation
    Uniform,
    // A buffer whose contents can be dynamic
    Storage,
    // A buffer whose purpose is to be used and then deleted shortly afterwards
    Oneshot,
}

pub(crate) struct BufferInfo {
    pub buffer: wgpu::Buffer,
    pub num_items: usize,
}

#[derive(Clone)]
pub struct InnerBufferConfiguration {
    pub buffer_type: BufferType,
    pub allow_write: bool,
    pub allow_read: bool,
}
pub struct Buffer {
    pub(crate) buffer: BufferInfo,
    pub(crate) bind_group: Option<BindGroup>,
    pub(crate) config: InnerBufferConfiguration,
}

impl Buffer {
    pub fn bind_group_layout(framework: &Framework) -> BindGroupLayout {
        framework
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Buffer BindGroup Layour"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            })
    }
}

impl From<BufferType> for BufferUsages {
    fn from(buffer_type: BufferType) -> Self {
        match buffer_type {
            BufferType::Vertex => BufferUsages::VERTEX,
            BufferType::Uniform => BufferUsages::UNIFORM,
            BufferType::Storage => BufferUsages::STORAGE,
            BufferType::Oneshot => BufferUsages::empty(),
        }
    }
}

pub enum BufferInitialSetup<'create, T>
where
    T: bytemuck::Pod + bytemuck::Zeroable,
{
    Data(&'create Vec<T>),
    Size(u64),
    Count(usize),
}

pub struct BufferConfiguration<'create, T>
where
    T: bytemuck::Pod + bytemuck::Zeroable,
{
    pub initial_setup: BufferInitialSetup<'create, T>,
    pub buffer_type: BufferType,
    pub allow_write: bool,
    pub allow_read: bool,
}

pub(crate) fn recreate_buffer<T>(
    framework: &Framework,
    data: &BufferInitialSetup<T>,
    config: &InnerBufferConfiguration,
) -> BufferInfo
where
    T: bytemuck::Pod + bytemuck::Zeroable,
{
    let buffer_usage: BufferUsages = config.buffer_type.into();
    let usage: BufferUsages = buffer_usage
        | if config.allow_write {
            BufferUsages::COPY_DST
        } else {
            BufferUsages::empty()
        }
        | if config.allow_read {
            BufferUsages::COPY_SRC
        } else {
            BufferUsages::empty()
        };
    let (buffer, num_items) = match data {
        BufferInitialSetup::Data(data) => (
            framework
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: &bytemuck::cast_slice(data.as_slice()),
                    usage,
                }),
            data.as_slice().len(),
        ),
        BufferInitialSetup::Size(initial_size) => (
            framework.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: *initial_size,
                usage,
                mapped_at_creation: false,
            }),
            1,
        ),
        BufferInitialSetup::Count(nums) => (
            framework.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: (std::mem::size_of::<T>() * nums) as u64,
                usage,
                mapped_at_creation: false,
            }),
            *nums,
        ),
    };
    BufferInfo { buffer, num_items }
}

impl Buffer {
    pub(crate) fn new<T>(
        framework: &Framework,
        initial_configuration: BufferConfiguration<T>,
    ) -> Self
    where
        T: bytemuck::Pod + bytemuck::Zeroable,
    {
        let configuration = InnerBufferConfiguration {
            allow_read: initial_configuration.allow_read,
            allow_write: initial_configuration.allow_write,
            buffer_type: initial_configuration.buffer_type,
        };
        let buffer = recreate_buffer(
            framework,
            &initial_configuration.initial_setup,
            &configuration,
        );
        let bind_group = if configuration.buffer_type == BufferType::Uniform {
            let bind_group = framework
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Buffer BindGroup"),
                    layout: &Buffer::bind_group_layout(framework),
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            buffer.buffer.as_entire_buffer_binding(),
                        ),
                    }],
                });
            Some(bind_group)
        } else {
            None
        };
        Buffer {
            buffer: buffer,
            config: configuration,
            bind_group,
        }
    }

    pub(crate) fn inner_buffer(&self) -> &wgpu::Buffer {
        &self.buffer.buffer
    }

    pub(crate) fn read_all_sync(&self, framework: &'_ Framework) -> Vec<u8> {
        let device = &framework.device;
        let out_slice = self.buffer.buffer.slice(..);
        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
        out_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        device.poll(wgpu::Maintain::Wait);
        pollster::block_on(rx.receive()).unwrap().unwrap();

        let data = out_slice.get_mapped_range();
        data.iter().map(|b| *b).collect()
    }

    pub(crate) fn read_region(
        &self,
        framework: &'_ Framework,
        begin_and_size: (u64, u64),
    ) -> Vec<u8> {
        let (begin, size) = begin_and_size;
        let (tx, rx) = std::sync::mpsc::channel();
        let buffer_slice = self.inner_buffer().slice(begin..begin + size);
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| {
            tx.send(v).unwrap();
        });
        framework.device.poll(wgpu::Maintain::Wait);
        if let Err(e) = rx.recv() {
            panic!("While reading texture pixel: {e}");
        }
        let mapped_range = buffer_slice.get_mapped_range();
        let data = mapped_range.iter().map(|b| *b).collect();
        drop(mapped_range);
        self.inner_buffer().unmap();
        data
    }
    pub(crate) fn entire_slice(&self) -> BufferSlice {
        self.buffer.buffer.slice(..)
    }
}
