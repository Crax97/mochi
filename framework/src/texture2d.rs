use std::num::{NonZeroU32, NonZeroU8};
use wgpu::{BindGroup, Color, Extent3d, ImageCopyBuffer, ImageDataLayout};

use crate::Framework;

pub struct GpuImageData {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub padded_width: u32,
}

pub struct Texture2d {
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

pub struct Texture2dConfiguration {
    pub debug_name: Option<String>,
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
    pub allow_cpu_write: bool,
    pub allow_cpu_read: bool,
    pub allow_use_as_render_target: bool,
}

impl Texture2d {
    pub(crate) fn new(framework: &Framework, config: Texture2dConfiguration) -> Self {
        let enable_if = |cond, feature| {
            if cond {
                feature
            } else {
                wgpu::TextureUsages::empty()
            }
        };

        let texture = framework.device.create_texture(&wgpu::TextureDescriptor {
            label: config.debug_name.as_deref(),
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
            label: Some("Framework Texture view"),
            format: Some(config.format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        let sampler = framework.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Framework Texture sampler"),
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
    pub fn sample_pixel(&self, x: u32, y: u32, framework: &Framework) -> wgpu::Color {
        let texture_region = wgpu::ImageCopyTexture {
            texture: &self.texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x, y, z: 0 },
            aspect: wgpu::TextureAspect::All,
        };
        let oneshot_buffer =
            framework.allocate_typed_buffer(crate::TypedBufferConfiguration::<u8> {
                initial_setup: crate::typed_buffer::BufferInitialSetup::Size(
                    wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64,
                ),
                buffer_type: crate::BufferType::Oneshot,
                allow_write: true,
                allow_read: true,
            });
        let mut encoder =
            framework
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Gpu -> pixel"),
                });
        encoder.copy_texture_to_buffer(
            texture_region,
            ImageCopyBuffer {
                buffer: oneshot_buffer.inner_buffer(),
                layout: ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT),
                    rows_per_image: NonZeroU32::new(1),
                },
            },
            Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        framework.queue.submit(std::iter::once(encoder.finish()));
        let color_bytes = oneshot_buffer.read_region((0, 4));
        Color {
            r: (color_bytes[0] as f64) / 255.0,
            g: (color_bytes[1] as f64) / 255.0,
            b: (color_bytes[2] as f64) / 255.0,
            a: (color_bytes[3] as f64) / 255.0,
        }
    }

    pub fn write_data(&self, bytes: &[u8], framework: &Framework) {
        self.write_region(bytes, (0, 0, self.width, self.height), framework);
    }

    pub fn write_region(
        &self,
        region_bytes: &[u8],
        region_rect: (u32, u32, u32, u32),
        framework: &Framework,
    ) {
        let (x, y, w, h) = region_rect;
        let total_size_to_copy = w * h * 4;
        let buffer_offset = x * y * 4;
        assert!(total_size_to_copy as usize <= region_bytes.len());

        let texture_region = wgpu::ImageCopyTexture {
            texture: &self.texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x, y, z: 0 },
            aspect: wgpu::TextureAspect::All,
        };

        framework.queue.write_texture(
            texture_region,
            region_bytes,
            wgpu::ImageDataLayout {
                offset: buffer_offset as u64,
                bytes_per_row: NonZeroU32::new(self.width * 4),
                rows_per_image: NonZeroU32::new(self.height),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        )
    }

    pub fn read_data(&self, framework: &Framework) -> GpuImageData {
        let mut encoder =
            framework
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("copy texture to buffer"),
                });

        let channels = 4;
        let unpadded_width = self.width * channels;
        let pad_bytes = (wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
            - (unpadded_width % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT))
            % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_width = unpadded_width + pad_bytes;
        let oneshot_buffer = framework.allocate_typed_buffer(crate::TypedBufferConfiguration {
            initial_setup: crate::typed_buffer::BufferInitialSetup::Size::<u8>(
                (padded_width * self.height) as u64,
            ),
            buffer_type: crate::BufferType::Oneshot,
            allow_write: true,
            allow_read: true,
        });
        encoder.copy_texture_to_buffer(
            self.texture.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer: &oneshot_buffer.inner_buffer(),
                layout: ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(padded_width),
                    rows_per_image: std::num::NonZeroU32::new(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
        framework.queue.submit(std::iter::once(encoder.finish()));

        let bytes = oneshot_buffer.read_all_sync();
        GpuImageData {
            data: bytes,
            width: self.width,
            height: self.height,
            padded_width: padded_width / channels,
        }
    }

    pub fn texture_view(&self) -> &wgpu::TextureView {
        &self.texture_view
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}
