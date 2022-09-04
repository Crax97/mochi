use crate::framework::Framework;
use cgmath::num_traits::ToPrimitive;
use wgpu::{
    BindGroup, Color, ImageDataLayout, Sampler, SamplerDescriptor, Texture, TextureDescriptor,
    TextureView, TextureViewDescriptor, TextureViewDimension,
};

pub struct BitmapLayerConfiguration {
    pub label: String,
    pub width: u32,
    pub initial_background_color: [f32; 4],
    pub height: u32,
}
pub struct BitmapLayer {
    texture: Texture,
    rgba_texture_view: TextureView,
    sampler: Sampler,
    bind_group: BindGroup,
    configuration: BitmapLayerConfiguration,
}

impl BitmapLayer {
    pub fn new(framework: &Framework, configuration: BitmapLayerConfiguration) -> Self {
        let bytes: Vec<u32> = (1..(configuration.width * configuration.height) + 1)
            .map(|_| {
                let bg = configuration.initial_background_color;
                let r = (bg[0].clamp(0.0, 1.0) * 255.0).to_u8().unwrap();
                let g = (bg[1].clamp(0.0, 1.0) * 255.0).to_u8().unwrap();
                let b = (bg[2].clamp(0.0, 1.0) * 255.0).to_u8().unwrap();
                let a = (bg[3].clamp(0.0, 1.0) * 255.0).to_u8().unwrap();
                u32::from_le_bytes([r, g, b, a])
            })
            .collect();
        let bytes = bytemuck::cast_slice(&bytes);
        Self::new_from_bytes(framework, &bytes, configuration)
    }

    pub fn new_from_bytes(
        framework: &Framework,
        bytes: &[u8],
        configuration: BitmapLayerConfiguration,
    ) -> Self {
        let dimension = wgpu::TextureDimension::D2;
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;

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
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
        });

        framework.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &&texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bytemuck::cast_slice(&bytes),
            ImageDataLayout {
                bytes_per_row: std::num::NonZeroU32::new(configuration.width * 4),
                rows_per_image: std::num::NonZeroU32::new(configuration.height),
                offset: 0,
            },
            size,
        );

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

        let bind_group_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Final render group layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let bind_group = framework
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Final Draw render pass"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&rgba_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });

        Self {
            texture,
            rgba_texture_view,
            configuration,
            bind_group,
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

    pub(crate) fn binding_group(&self) -> &BindGroup {
        &&self.bind_group
    }
}
