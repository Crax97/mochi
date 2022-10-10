use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use cgmath::{point2, point3};
use log::*;
use uuid::Uuid;
use wgpu::*;

use crate::{
    asset_library, AllocatedAsset, Asset, AssetId, AssetMap, AssetsLibrary, Mesh,
    MeshConstructionDetails, Texture2d, Texture2dConfiguration, Vertex,
};

use super::buffer::{Buffer, BufferConfiguration};

pub type TextureId = AssetId<Texture2d>;
type TextureMap = AssetMap<Texture2d>;
type AllocatedTexture = AllocatedAsset<Texture2d>;

pub struct Framework {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub asset_library: AssetsLibrary,

    allocated_textures: TextureMap,
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
    fn construct_initial_assets(f: &mut Framework) {
        let quad_mesh_vertices = [
            Vertex {
                position: point3(-1.0, 1.0, 0.0),
                tex_coords: point2(0.0, 1.0),
            },
            Vertex {
                position: point3(1.0, 1.0, 0.0),
                tex_coords: point2(1.0, 1.0),
            },
            Vertex {
                position: point3(-1.0, -1.0, 0.0),
                tex_coords: point2(0.0, 0.0),
            },
            Vertex {
                position: point3(1.0, -1.0, 0.0),
                tex_coords: point2(1.0, 0.0),
            },
        ]
        .into();

        let indices = [0u16, 1, 2, 2, 1, 3].into();
        let quad_mesh = Mesh::new(
            &f,
            MeshConstructionDetails {
                vertices: quad_mesh_vertices,
                indices,
                allow_editing: false,
            },
        );

        f.asset_library
            .add_mesh(asset_library::mesh_names::QUAD, quad_mesh);
    }

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
        let asset_library = AssetsLibrary::new();
        let mut framework = Framework {
            instance,
            adapter,
            device,
            queue,
            asset_library,
            allocated_textures: Arc::new(Mutex::new(HashMap::new())),
        };
        Framework::construct_initial_assets(&mut framework);
        Ok(framework)
    }

    pub fn allocate_typed_buffer<BufferType: bytemuck::Pod + bytemuck::Zeroable>(
        &'a self,
        configuration: BufferConfiguration<BufferType>,
    ) -> Buffer {
        Buffer::new(self, configuration)
    }

    pub fn allocate_texture2d(
        &self,
        tex_info: Texture2dConfiguration,
        initial_data: Option<&[u8]>,
    ) -> TextureId {
        let tex = Texture2d::new(&self, tex_info);
        if let Some(data) = initial_data {
            tex.write_data(data, &self);
        }
        let alloc_texture = AllocatedTexture::new(tex);
        let tex_id = TextureId::new(self.allocated_textures.clone());
        self.allocated_textures
            .lock()
            .unwrap()
            .insert(tex_id.0.clone(), alloc_texture);
        tex_id
    }

    pub fn texture2d(&self, id: &TextureId) -> Asset<Texture2d> {
        self.allocated_textures
            .lock()
            .unwrap()
            .get(&id.0)
            .expect("Failed to find given texture2d")
            .asset
            .clone()
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
