use std::num::NonZeroU8;

use wgpu::{BindGroup, Sampler, Texture, TextureAspect, TextureView};

use crate::Framework;

pub struct DepthStencilTextureConfiguration<'a> {
    pub debug_name: Option<&'a str>,
    pub width: u32,
    pub height: u32,
    pub is_stencil: bool,
}

pub struct DepthStencilTexture {
    pub(crate) depth_stencil_texture: Texture,

    pub(crate) depth_view: TextureView,
    pub(crate) depth_sampler: Sampler,
    pub(crate) depth_bind_group: BindGroup,

    pub(crate) stencil_view: TextureView,
    pub(crate) stencil_sampler: Sampler,
    pub(crate) stencil_bind_group: BindGroup,

    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) is_stencil: bool,
}

impl DepthStencilTexture {
    pub(crate) fn new(framework: &Framework, config: DepthStencilTextureConfiguration) -> Self {
        let format = wgpu::TextureFormat::Depth24PlusStencil8;

        let depth_stencil_texture = framework.device.create_texture(&wgpu::TextureDescriptor {
            label: config.debug_name,
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
        });
        let make_view_and_bind_group = |aspect: TextureAspect| {
            let texture_view = depth_stencil_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(
                    format!(
                        "Framework DepthStencil Texture view, aspect {:?} label {:?}",
                        aspect, config.debug_name
                    )
                    .as_str(),
                ),
                format: Some(format),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });
            let sampler = framework.device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some(
                    format!(
                        "Framework DepthStencil Sampler, aspect {:?} label {:?}",
                        aspect, config.debug_name
                    )
                    .as_str(),
                ),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                lod_min_clamp: 0.0,
                lod_max_clamp: 0.0,
                compare: None,
                anisotropy_clamp: NonZeroU8::new(1),
                border_color: None,
            });

            let texture_bind_group_layout =
                framework
                    .device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: Some("Depth Texture Bindg layout"),
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    sample_type: wgpu::TextureSampleType::Float {
                                        filterable: true,
                                    },
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    multisampled: false,
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                                count: None,
                            },
                        ],
                    });
            let bind_group = framework
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(
                        format!(
                            "Framework DepthStencil BindGroup, aspect {:?} label {:?}",
                            aspect, config.debug_name
                        )
                        .as_str(),
                    ),
                    layout: &texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                });
            (texture_view, sampler, bind_group)
        };

        let (depth_view, depth_sampler, depth_bind_group) =
            make_view_and_bind_group(TextureAspect::DepthOnly);
        let (stencil_view, stencil_sampler, stencil_bind_group) =
            make_view_and_bind_group(TextureAspect::StencilOnly);

        Self {
            depth_stencil_texture,
            depth_view,
            depth_sampler,
            depth_bind_group,
            stencil_view,
            stencil_sampler,
            stencil_bind_group,
            width: config.width,
            height: config.height,
            is_stencil: config.is_stencil,
        }
    }
}
