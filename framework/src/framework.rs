use anyhow::Result;
use log::*;

use wgpu::*;

use crate::{
    buffer::BufferInitialSetup,
    shader::{Shader, ShaderCompiler, ShaderCreationInfo},
    AssetId, AssetMap, AssetsLibrary, DepthStencilTexture2D, GpuDepthStencilTexture2D,
    GpuRgbaTexture2D, GpuTexture, Mesh, MeshConstructionDetails, RgbaTexture2D, RgbaU8, Texel,
    TextureConfiguration,
};

use super::buffer::{Buffer, BufferConfiguration};

pub type TextureId = AssetId<GpuRgbaTexture2D>;
type TextureMap = AssetMap<GpuRgbaTexture2D>;

pub type DepthStencilTextureId = AssetId<GpuDepthStencilTexture2D>;
type DepthStencilTextureMap = AssetMap<GpuDepthStencilTexture2D>;

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
    pub shader_compiler: ShaderCompiler,

    pub(crate) allocated_textures: TextureMap,
    pub(crate) allocated_depth_stencil_textures: DepthStencilTextureMap,
    pub(crate) allocated_buffers: BufferMap,
    pub(crate) allocated_shaders: ShaderMap,
    pub(crate) allocated_meshes: MeshMap,
}

impl Framework {
    fn build_shader_compiler() -> ShaderCompiler {
        let mut shader_compiler = ShaderCompiler::new();
        shader_compiler
            .define(
                "common_definitions",
                include_str!("shader/default_shaders/common_definitions.wgsl"),
            )
            .expect("Failed to compile common definitions");
        shader_compiler
            .define(
                "2d_definitions",
                include_str!("shader/default_shaders/2d_definitions.wgsl"),
            )
            .expect("Failed to compile 2d definitions");
        shader_compiler
            .define(
                "2d_transformations",
                include_str!("shader/default_shaders/2d_transformations.wgsl"),
            )
            .expect("Failed to compile 2d transformations functions");
        shader_compiler
    }

    pub(crate) fn buffer_oneshot(&self, config: BufferConfiguration<u8>) -> Buffer {
        Buffer::new(self, config)
    }
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
        let shader_compiler = Framework::build_shader_compiler();
        let framework = Framework {
            instance,
            adapter,
            device,
            queue,
            asset_library,
            allocated_textures: AssetMap::new(),
            allocated_depth_stencil_textures: AssetMap::new(),
            allocated_buffers: AssetMap::new(),
            allocated_shaders: AssetMap::new(),
            allocated_meshes: AssetMap::new(),
            shader_compiler,
        };
        Ok(framework)
    }

    pub fn allocate_typed_buffer<BufferType: bytemuck::Pod + bytemuck::Zeroable>(
        &mut self,
        configuration: BufferConfiguration<BufferType>,
    ) -> BufferId {
        let buffer = Buffer::new(self, configuration);

        self.allocated_buffers.insert(buffer)
    }

    pub(crate) fn buffer(&self, id: &BufferId) -> &Buffer {
        self.allocated_buffers.get(id)
    }
    pub(crate) fn buffer_mut(&mut self, id: &BufferId) -> &mut Buffer {
        self.allocated_buffers.get_mut(id)
    }

    pub fn allocate_texture2d<'r>(
        &mut self,
        texture: RgbaTexture2D,
        config: TextureConfiguration,
    ) -> TextureId {
        let gpu_tex = GpuTexture::new(texture, config, self);
        self.allocated_textures.insert(gpu_tex)
    }

    pub(crate) fn texture2d(&self, id: &TextureId) -> &GpuTexture<RgbaU8, RgbaTexture2D> {
        self.allocated_textures.get(id)
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

    pub fn update_asset_maps(&mut self) {
        self.allocated_buffers.update();
        self.allocated_shaders.update();
        self.allocated_textures.update();
        self.allocated_depth_stencil_textures.update();
        self.allocated_meshes.update();
    }
    pub fn allocate_mesh(&mut self, construction_info: MeshConstructionDetails) -> MeshId {
        let mesh = Mesh::new(self, construction_info);
        self.allocated_meshes.insert(mesh)
    }

    pub fn mesh<'r>(&'r self, id: &MeshId) -> &'r Mesh {
        self.allocated_meshes.get(id)
    }
}

// Shaders
impl<'a> Framework {
    pub fn create_shader(&mut self, info: ShaderCreationInfo) -> ShaderId {
        let shader = Shader::new(&self, info);
        self.allocated_shaders.insert(shader)
    }

    pub(crate) fn shader(&self, id: &ShaderId) -> &Shader {
        self.allocated_shaders.get(id)
    }
}
// Buffer
impl<'a> Framework {
    pub fn buffer_write_sync<T: bytemuck::Pod + bytemuck::Zeroable>(
        &mut self,
        id: &BufferId,
        data: Vec<T>,
    ) {
        // TODO: fix this ugly shit
        let length = data.as_slice().len();
        let current_items = { self.buffer(id).buffer.num_items };

        if length > current_items {
            let config = self.buffer(id).config.clone();
            let new_buf =
                crate::buffer::recreate_buffer(self, &BufferInitialSetup::Data(&data), &config);

            let buf_mut = self.buffer_mut(id);
            buf_mut.buffer = new_buf;
        }
        {
            let buffer = self.buffer_mut(id);
            buffer.buffer.num_items = data.len();
        }
        {
            let buffer = self.buffer(id);
            let buffer = &buffer.buffer.buffer;
            self.queue
                .write_buffer(&buffer, 0, &bytemuck::cast_slice(&data.as_slice()));
        }
    }
}

// Texture2D
impl<'a> Framework {
    pub fn texture2d_dimensions(&self, id: &TextureId) -> (u32, u32) {
        (self.texture2d_width(id), self.texture2d_height(id))
    }

    pub fn texture2d_width(&self, id: &TextureId) -> u32 {
        self.texture2d(id).width()
    }
    pub fn texture2d_height(&self, id: &TextureId) -> u32 {
        self.texture2d(id).height()
    }
    pub fn texture2d_format(&self, id: &TextureId) -> TextureFormat {
        RgbaU8::wgpu_texture_format()
    }

    pub fn texture2d_sample_pixel(&self, id: &TextureId, x: u32, y: u32) -> wgpu::Color {
        self.texture2d(id)
            .sample((x, y), self)
            .unwrap()
            .wgpu_color()
    }
    pub fn texture2d_read_data(&self, id: &TextureId) -> RgbaTexture2D {
        self.texture2d(id).read_data(self).unwrap()
    }
    pub fn texture2d_copy_subregion(
        &mut self,
        id: &TextureId,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> TextureId {
        let new_texture = self
            .texture2d(id)
            .clone_subregion((x, y), (width, height), self);

        self.allocated_textures.insert(new_texture)
    }
}

// DepthStencilTexture
impl<'a> Framework {
    pub fn allocate_depth_stencil_texture(
        &mut self,
        texture: DepthStencilTexture2D,
        config: TextureConfiguration,
    ) -> DepthStencilTextureId {
        let depth_stencil = GpuDepthStencilTexture2D::new(texture, config, self);
        self.allocated_depth_stencil_textures.insert(depth_stencil)
    }

    pub fn depth_stencil_texture(&self, id: &DepthStencilTextureId) -> &GpuDepthStencilTexture2D {
        self.allocated_depth_stencil_textures.get(id)
    }
}
