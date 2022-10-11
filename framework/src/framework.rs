use std::{
    cell::RefCell,
    collections::HashMap,
    ops::RangeFull,
    rc::Rc,
    sync::{Arc, RwLock},
};

use anyhow::Result;
use cgmath::{point2, point3};
use log::*;

use wgpu::*;

use crate::{
    asset_library, texture2d::GpuImageData, AssetId, AssetMap, AssetRef, AssetRefMut,
    AssetsLibrary, Mesh, MeshConstructionDetails, Texture2d, Texture2dConfiguration, Vertex,
};

use super::buffer::{Buffer, BufferConfiguration};

pub type TextureId = AssetId;
type TextureMap = AssetMap<Texture2d>;

pub type BufferId = AssetId;
type BufferMap = AssetMap<Buffer>;

pub struct Framework {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub asset_library: AssetsLibrary,

    allocated_textures: TextureMap,
    allocated_buffers: BufferMap,
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

    pub fn new(device_descriptor: &DeviceDescriptor<'a>) -> Result<Self> {
        let instance = wgpu::Instance::new(Backends::all());
        let adapter = pollster::block_on(async {
            instance
                .request_adapter(&RequestAdapterOptions {
                    power_preference: PowerPreference::HighPerformance,
                    compatible_surface: None,
                    force_fallback_adapter: false,
                })
                .await
        })
        .ok_or(AdapterCreationError)?;
        let (device, queue) =
            pollster::block_on(async { adapter.request_device(&device_descriptor, None).await })?;
        let asset_library = AssetsLibrary::new();
        let mut framework = Framework {
            instance,
            adapter,
            device,
            queue,
            asset_library,
            allocated_textures: Rc::new(RefCell::new(HashMap::new())),
            allocated_buffers: Rc::new(RefCell::new(HashMap::new())),
        };
        Framework::construct_initial_assets(&mut framework);
        Ok(framework)
    }

    pub fn allocate_typed_buffer<BufferType: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        configuration: BufferConfiguration<BufferType>,
    ) -> BufferId {
        let buffer = Buffer::new(self, configuration);

        let buf_id = BufferId::new();
        self.allocated_buffers
            .borrow_mut()
            .insert(buf_id.0.clone(), buffer);
        buf_id
    }

    pub fn buffer_bind_group(&self, id: &BufferId) -> &BindGroup {
        todo!()
    }
    pub fn buffer_slice<'r>(&'r self, id: &BufferId, range: RangeFull) -> BufferSlice {
        todo!()
    }
    pub fn buffer_elem_count<'r>(&'r self, id: &BufferId) -> u32 {
        todo!()
    }

    pub fn buffer_write_sync<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        id: &BufferId,
        data: Vec<T>,
    ) {
        todo!()
    }

    pub(crate) fn buffer<'r>(&'r self, id: &BufferId) -> AssetRef<'r, Buffer> {
        AssetRef {
            in_ref: self.allocated_buffers.borrow(),
            id: id.clone(),
        }
    }
    pub(crate) fn buffer_mut<'r>(&'r self, id: &BufferId) -> AssetRefMut<'r, Buffer> {
        AssetRefMut {
            in_ref: self.allocated_buffers.borrow_mut(),
            id: id.clone(),
        }
    }

    pub fn allocate_texture2d<'r>(
        &self,
        tex_info: Texture2dConfiguration,
        initial_data: Option<&[u8]>,
    ) -> TextureId {
        let tex = Texture2d::new(&self, tex_info);
        if let Some(data) = initial_data {
            tex.write_data(data, &self);
        }
        let tex_id = TextureId::new();
        self.allocated_textures
            .borrow_mut()
            .insert(tex_id.0.clone(), tex);
        tex_id
    }

    pub(crate) fn texture2d<'r>(&'r self, id: &TextureId) -> AssetRef<'r, Texture2d> {
        AssetRef {
            in_ref: self.allocated_textures.borrow(),
            id: id.clone(),
        }
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

// Texture2D
impl<'a> Framework {
    pub fn texture2d_bind_group(&self, id: &TextureId) -> &BindGroup {
        todo!()
    }

    pub fn texture2d_dimensions(&self, id: &TextureId) -> (u32, u32) {
        (self.texture2d_width(id), self.texture2d_height(id))
    }

    pub fn texture2d_width(&self, id: &TextureId) -> u32 {
        todo!()
    }
    pub fn texture2d_height(&self, id: &TextureId) -> u32 {
        todo!()
    }
    pub fn texture2d_format(&self, id: &TextureId) -> TextureFormat {
        todo!()
    }

    pub fn texture2d_sample_pixel(&self, id: &TextureId, x: u32, y: u32) -> wgpu::Color {
        todo!()
    }
    pub fn texture2d_texture_view(&self, id: &TextureId) -> &TextureView {
        todo!()
    }
    pub fn texture2d_read_data(&self, id: &TextureId) -> GpuImageData {
        todo!()
    }
    pub fn texture2d_read_subregion(
        &self,
        id: &TextureId,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> TextureId {
        todo!()
    }
}
