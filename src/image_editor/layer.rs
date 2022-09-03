use crate::framework::Framework;
use wgpu::{
    Sampler, SamplerDescriptor, Texture, TextureDescriptor, TextureView, TextureViewDescriptor,
    TextureViewDimension,
};

pub struct LayerConfiguration {
    pub label: String,
    pub width: u32,
    pub height: u32,
    pub texture_format: wgpu::TextureFormat,
}
pub struct Layer {
    texture: Texture,
    rgba_texture_view: TextureView,
    sampler: Sampler,
    configuration: LayerConfiguration,
}

impl Layer {
    pub fn new(framework: &Framework, configuration: LayerConfiguration) -> Self {
        let dimension = wgpu::TextureDimension::D2;
        let format = configuration.texture_format;

        let size = wgpu::Extent3d {
            width: configuration.width,
            height: configuration.height,
            depth_or_array_layers: 1,
        };

        let texture = framework.device.create_texture(&TextureDescriptor {
            label: Some(format!("Layer {}", &configuration.label).as_ref()),
            dimension,
            format,
            size,
            mip_level_count: 1,
            sample_count: 1,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC,
        });
        let rgba_texture_view = texture.create_view(&TextureViewDescriptor {
            label: Some(format!("Layer View {}", &configuration.label).as_ref()),
            dimension: Some(TextureViewDimension::D2),
            format: Some(format),
            aspect: wgpu::TextureAspect::All,
            array_layer_count: None,
            base_array_layer: 0,
            base_mip_level: 0,
            mip_level_count: None,
        });

        let sampler = framework.device.create_sampler(&SamplerDescriptor {
            label: Some(format!("Layer Sampler {}", &configuration.label).as_ref()),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Layer {
            texture,
            rgba_texture_view,
            configuration,
            sampler,
        }
    }

    pub fn texture_view(&self) -> &TextureView {
        &self.rgba_texture_view
    }

    pub(crate) fn texture(&self) -> &Texture {
        &self.texture
    }

    pub(crate) fn sampler(&self) -> &Sampler {
        &self.sampler
    }
}
