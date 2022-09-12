use anyhow::Result;
use as_slice::AsSlice;
use log::*;
use wgpu::*;

use super::typed_buffer::{TypedBuffer, TypedBufferConfiguration};

pub struct Framework {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

#[derive(Debug)]
pub struct AdapterCreationError;

impl std::fmt::Display for AdapterCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to create a device: see the logs for what caused this problem.")?;
        Ok(())
    }
}

impl std::error::Error for AdapterCreationError {}

impl<'a> Framework {
    pub async fn new(device_descriptor: &DeviceDescriptor<'a>) -> Result<Self> {
        let instance = wgpu::Instance::new(Backends::all());
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(AdapterCreationError)?;
        let (device, queue) = adapter.request_device(&device_descriptor, None).await?;
        Ok(Framework {
            instance,
            adapter,
            device,
            queue,
        })
    }

    pub fn allocate_typed_buffer<BufferType: bytemuck::Pod + bytemuck::Zeroable>(
        &'a self,
        configuration: TypedBufferConfiguration<BufferType>,
    ) -> TypedBuffer<'a> {
        TypedBuffer::new(self, configuration)
    }

    pub fn log_info(&self) {
        let device_info = self.adapter.get_info();
        let backend_string = match device_info.backend {
            Backend::Empty => unreachable!(),
            Backend::Vulkan => "Vulkan",
            Backend::Metal => "Metal",
            Backend::Dx12 => "DirectX 12",
            Backend::Dx11 => "DirectX 11",
            Backend::Gl => "OpenGL",
            Backend::BrowserWebGpu => "WebGPU",
        };
        info!(
            "Created a new framework instance, using device {}",
            device_info.name
        );
        info!("\tUsing backend {}", backend_string);
    }
}
