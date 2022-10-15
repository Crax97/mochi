use std::{
    cell::RefCell,
    rc::Rc,
};

use anyhow::Result;
use log::*;

use wgpu::*;

use crate::{
    shader::{Shader, ShaderCreationInfo},
    texture2d::GpuImageData,
    AssetId, AssetMap, AssetRef, AssetRefMut, AssetsLibrary, InnerAssetMap, Mesh,
    MeshConstructionDetails, Texture2d, Texture2dConfiguration,
};

use super::buffer::{Buffer, BufferConfiguration};

pub type TextureId = AssetId<Texture2d>;
type TextureMap = AssetMap<Texture2d>;

pub type BufferId = AssetId<Buffer>;
type BufferMap = AssetMap<Buffer>;

pub type MeshId = AssetId<Mesh>;
type MeshMap = AssetMap<Mesh>;

pub type ShaderId = AssetId<Shader>;
type ShaderMap = AssetMap<Shader>;

pub struct Framework {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub asset_library: AssetsLibrary,

    allocated_textures: TextureMap,
    allocated_buffers: BufferMap,
    allocated_shaders: ShaderMap,
    allocated_meshes: MeshMap,
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
        let framework = Framework {
            instance,
            adapter,
            device,
            queue,
            asset_library,
            allocated_textures: Rc::new(RefCell::new(InnerAssetMap::new())),
            allocated_buffers: Rc::new(RefCell::new(InnerAssetMap::new())),
            allocated_shaders: Rc::new(RefCell::new(InnerAssetMap::new())),
            allocated_meshes: Rc::new(RefCell::new(InnerAssetMap::new())),
        };
        Ok(framework)
    }

    pub fn allocate_typed_buffer<BufferType: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        configuration: BufferConfiguration<BufferType>,
    ) -> BufferId {
        let buffer = Buffer::new(self, configuration);

        self.allocated_buffers.borrow_mut().insert(buffer)
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
        self.allocated_textures.borrow_mut().insert(tex)
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

    pub fn create_shader(&self, info: ShaderCreationInfo) -> ShaderId {
        let shader = Shader::new(&self, info);
        self.allocated_shaders.borrow_mut().insert(shader)
    }

    pub fn update_asset_maps(&self) {
        self.allocated_buffers.borrow_mut().update();
        self.allocated_shaders.borrow_mut().update();
        self.allocated_textures.borrow_mut().update();
    }
    pub fn allocate_mesh(&self, construction_info: MeshConstructionDetails) -> MeshId {
        let mesh = Mesh::new(self, construction_info);
        self.allocated_meshes.borrow_mut().insert(mesh)
    }

    pub fn mesh<'r>(&'r self, id: &MeshId) -> AssetRef<'r, Mesh> {
        AssetRef {
            in_ref: self.allocated_meshes.borrow(),
            id: id.clone(),
        }
    }
}

// Shaders
impl<'a> Framework {
    pub(crate) fn shader(&self, id: &ShaderId) -> AssetRef<Shader> {
        AssetRef {
            in_ref: self.allocated_shaders.borrow(),
            id: id.clone(),
        }
    }
}
// Buffer
impl<'a> Framework {
    pub fn buffer_write_sync<T: bytemuck::Pod + bytemuck::Zeroable>(
        &self,
        id: &BufferId,
        data: Vec<T>,
    ) {
        self.buffer_mut(id).write_sync(self, &data);
    }
}

// Texture2D
impl<'a> Framework {
    pub fn texture2d_dimensions(&self, id: &TextureId) -> (u32, u32) {
        (self.texture2d_width(id), self.texture2d_height(id))
    }

    pub fn texture2d_width(&self, id: &TextureId) -> u32 {
        self.texture2d(id).width
    }
    pub fn texture2d_height(&self, id: &TextureId) -> u32 {
        self.texture2d(id).height
    }
    pub fn texture2d_format(&self, id: &TextureId) -> TextureFormat {
        self.texture2d(id).format
    }

    pub fn texture2d_sample_pixel(&self, id: &TextureId, x: u32, y: u32) -> wgpu::Color {
        self.texture2d(id).sample_pixel(x, y, self)
    }
    pub fn texture2d_read_data(&self, id: &TextureId) -> GpuImageData {
        self.texture2d(id).read_data(self)
    }
    pub fn texture2d_copy_subregion(
        &self,
        id: &TextureId,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> TextureId {
        let format = { self.texture2d(id).format };
        let output_texture = self.allocate_texture2d(
            crate::Texture2dConfiguration {
                debug_name: Some("Tex Subregion".into()),
                width,
                height,
                format,
                allow_cpu_write: true,
                allow_cpu_read: true,
                allow_use_as_render_target: true,
            },
            None,
        );
        self.texture2d(id)
            .read_subregion_texture2d(x, y, width, height, &output_texture, self);
        output_texture
    }
}
