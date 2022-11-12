use anyhow::Result;
use log::*;

use wgpu::*;

use crate::{
    buffer::BufferInitialSetup,
    shader::{Shader, ShaderCompiler, ShaderCreationInfo},
    texture,
    texture2d::GpuImageData,
    AssetId, AssetMap, AssetsLibrary, DepthStencilTexture, DepthStencilTextureConfiguration,
    GpuRgbaTexture2D, GpuTexture, Mesh, MeshConstructionDetails, RgbaTexture2D, RgbaU8, Texel,
    TexelConversionError, Texture, Texture2dConfiguration,
};

use super::buffer::{Buffer, BufferConfiguration};

pub type TextureId = AssetId<GpuRgbaTexture2D>;
type TextureMap = AssetMap<GpuRgbaTexture2D>;

pub type DepthStencilTextureId = AssetId<DepthStencilTexture>;
type DepthStencilTextureMap = AssetMap<DepthStencilTexture>;

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
        tex_info: Texture2dConfiguration,
        initial_data: Option<&[u8]>,
    ) -> TextureId {
        let allow_cpu_write = tex_info.allow_cpu_write;
        let cpu_tex = if let Some(bytes) = initial_data {
            let texels: Result<Vec<RgbaU8>, TexelConversionError> = bytes
                .chunks(RgbaU8::channel_count() as usize * RgbaU8::channel_size_bytes() as usize)
                .map(|chunk| RgbaU8::from_bytes(bytes))
                .collect();
            let texels = texels.unwrap();
            RgbaTexture2D::from_texels(texels, (tex_info.width, tex_info.height)).unwrap()
        } else {
            RgbaTexture2D::empty((tex_info.width, tex_info.height))
        };
        let gpu_tex = GpuTexture::new(
            cpu_tex,
            texture::TextureConfiguration {
                label: tex_info.debug_name.as_deref(),
                usage: crate::TextureUsage {
                    cpu_write: tex_info.allow_cpu_write,
                    cpu_read: tex_info.allow_cpu_read,
                    use_as_render_target: tex_info.allow_use_as_render_target,
                },
                mip_count: None,
            },
            &self.device,
        );
        self.allocated_textures.insert(gpu_tex)
    }

    pub fn with_external_texture<F: FnMut(&TextureId, &mut Framework)>(
        &mut self,
        view: TextureView,
        mut f: F,
    ) -> TextureView {
        todo!()
        /*
        let tex = Texture2d::new_external(view);
        let id = self.allocated_textures.insert(tex);
        f(&id, self);
        self.take_external_texture2d_view(id)
        */
    }

    fn take_external_texture2d_view(&mut self, view: TextureId) -> TextureView {
        todo!()
        /*
        let tex = self.allocated_textures.take(view);
        match tex.tex_type {
            crate::texture2d::TextureType::Managed { .. } => {
                panic!("Cannot treat a managed texture2d as external!")
            }
            crate::texture2d::TextureType::External => tex.texture_view,
        }
        */
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
    pub fn texture2d_read_data(&self, id: &TextureId) -> GpuImageData {
        todo!()
    }
    pub fn texture2d_copy_subregion(
        &mut self,
        id: &TextureId,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> TextureId {
        let format = { RgbaU8::wgpu_texture_format() };
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
        config: DepthStencilTextureConfiguration,
    ) -> DepthStencilTextureId {
        let depth_stencil = DepthStencilTexture::new(&self, config);
        self.allocated_depth_stencil_textures.insert(depth_stencil)
    }

    pub fn depth_stencil_texture(&self, id: &DepthStencilTextureId) -> &DepthStencilTexture {
        self.allocated_depth_stencil_textures.get(id)
    }
}
