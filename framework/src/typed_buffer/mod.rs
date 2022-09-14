use wgpu::{util::DeviceExt, BufferUsages};

use crate::render_pass::PassBindble;

use super::framework::Framework;

#[derive(Copy, Clone, Debug)]
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
    config: InnerBufferConfiguration,
    owner_framework: &'framework Framework,
}

impl From<BufferType> for BufferUsages {
    fn from(buffer_type: BufferType) -> Self {
        match buffer_type {
            BufferType::Vertex => wgpu::BufferUsages::VERTEX,
            BufferType::Uniform => wgpu::BufferUsages::UNIFORM,
            BufferType::Storage => wgpu::BufferUsages::STORAGE,
            BufferType::Oneshot => wgpu::BufferUsages::empty(),
        }
    }
}

pub enum BufferInitialSetup<'create, T>
where
    T: bytemuck::Pod + bytemuck::Zeroable,
{
    Data(&'create Vec<T>),
    Size(u64),
}

pub struct TypedBufferConfiguration<'create, T>
where
    T: bytemuck::Pod + bytemuck::Zeroable,
{
    pub initial_setup: BufferInitialSetup<'create, T>,
    pub buffer_type: BufferType,
    pub allow_write: bool,
    pub allow_read: bool,
}

fn recreate_buffer<T>(
    data: &BufferInitialSetup<T>,
    config: &InnerBufferConfiguration,
    framework: &'_ Framework,
) -> BufferInfo
where
    T: bytemuck::Pod + bytemuck::Zeroable,
{
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
    };
    BufferInfo { buffer, num_items }
}

impl<'framework> TypedBuffer<'framework> {
    pub fn new<T>(
        framework: &'framework Framework,
        initial_configuration: TypedBufferConfiguration<T>,
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
            &initial_configuration.initial_setup,
            &configuration,
            framework,
        );
        TypedBuffer {
            buffer: buffer,
            config: configuration,
            owner_framework: framework,
        }
    }

    pub fn inner_buffer(&self) -> &wgpu::Buffer {
        &self.buffer.buffer
    }

    pub fn read_all_sync(&self) -> Vec<u8> {
        let device = &self.owner_framework.device;
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

    pub fn write_sync<T: bytemuck::Pod + bytemuck::Zeroable>(&mut self, data: &Vec<T>) {
        let queue = &self.owner_framework.queue;
        let length = data.as_slice().len();
        let current_items = self.buffer.num_items;

        if length > current_items {
            self.buffer = recreate_buffer(
                &BufferInitialSetup::Data(data),
                &self.config,
                self.owner_framework,
            );
        }
        self.buffer.num_items = data.len();
        let buffer = &self.buffer.buffer;
        queue.write_buffer(&buffer, 0, &bytemuck::cast_slice(&data.as_slice()));
    }

    pub fn binding_resource(&self) -> wgpu::BufferBinding {
        let buffer = &self.buffer.buffer;
        buffer.as_entire_buffer_binding()
    }

    pub fn elem_count(&self) -> usize {
        self.buffer.num_items
    }
}

impl<'framework> PassBindble for TypedBuffer<'framework> {
    fn bind<'s, 'call, 'pass>(&'s self, index: u32, pass: &'call mut wgpu::RenderPass<'pass>)
    where
        'pass: 'call,
        's: 'pass,
    {
        match self.config.buffer_type {
            BufferType::Vertex => {
                let buffer = &self.buffer.buffer;

                pass.set_vertex_buffer(index, buffer.slice(..));
            }
            BufferType::Uniform => {
                panic!("Uniform buffers should be set by using the associated bind group!")
            }
            BufferType::Storage => todo!(),
            BufferType::Oneshot => {
                panic!("Oneshot buffers aren't supposed to be bound!")
            }
        };
    }
}
