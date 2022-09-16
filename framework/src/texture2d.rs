use std::num::{NonZeroU32, NonZeroU8};

use wgpu::BindGroup;

use crate::{framework, render_pass::PassBindble, Framework};

pub struct Texture2d {
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

pub struct Texture2dConfiguration {
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
    pub allow_cpu_write: bool,
    pub allow_cpu_read: bool,
    pub allow_use_as_render_target: bool,
}

impl Texture2d {
    pub fn new(framework: &Framework, config: Texture2dConfiguration) -> Self {
        let enable_if = |cond, feature| {
            if cond {
                feature
            } else {
                wgpu::TextureUsages::empty()
            }
        };

        let texture = framework.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Document final render"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | enable_if(config.allow_cpu_read, wgpu::TextureUsages::COPY_SRC)
                | enable_if(config.allow_cpu_write, wgpu::TextureUsages::COPY_DST)
                | enable_if(
                    config.allow_use_as_render_target,
                    wgpu::TextureUsages::RENDER_ATTACHMENT,
                ),
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Document final render view for canvas"),
            format: Some(config.format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        let sampler = framework.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Document final render view for canvas"),
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
                    label: Some("Document final bind group layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
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
                label: Some("Final render texture bind group"),
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
        Self {
            width: config.width,
            height: config.height,
            texture,
            texture_view,
            sampler,
            bind_group,
        }
    }
}

impl Texture2d {
    pub fn write_data(&self, bytes: &[u8], framework: &Framework) {
        framework.queue.write_texture(
            self.texture.as_image_copy(),
            bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(self.width * 4),
                rows_per_image: NonZeroU32::new(self.height),
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        )
    }

    pub fn texture_view(&self) -> &wgpu::TextureView {
        &self.texture_view
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

impl PassBindble for Texture2d {
    fn bind<'s, 'call, 'pass>(&'s self, index: u32, pass: &'call mut wgpu::RenderPass<'pass>)
    where
        'pass: 'call,
        's: 'pass,
    {
        pass.set_bind_group(index, &self.bind_group, &[]);
    }
}
