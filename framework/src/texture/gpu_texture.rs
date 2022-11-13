use std::{marker::PhantomData, num::NonZeroU32};

use wgpu::{Extent3d, ImageCopyBuffer, ImageDataLayout, Origin3d, TextureDescriptor};

use crate::{
    BindingInfo, Framework, SamplingExtents, SamplingOrigin, TexelConversionError,
    TextureConfiguration, TextureUsage,
};

use super::{Texel, Texture};

pub struct GpuTexture<L: Texel, T: Texture<L>> {
    phant_data: PhantomData<(T, L)>,
    pub(crate) label: Option<String>,
    pub(crate) size: Extent3d,
    pub(crate) wgpu_texture: wgpu::Texture,
    pub(crate) usage: TextureUsage,
    pub(crate) mip_count: Option<u32>,

    pub(crate) binding_infos: Vec<BindingInfo>,
}

impl<L: Texel, T: Texture<L>> GpuTexture<L, T> {
    pub(crate) fn new<'a>(
        texture: T,
        config: TextureConfiguration<'a>,
        framework: &Framework,
    ) -> GpuTexture<L, T> {
        let size = Extent3d {
            width: texture.width(),
            height: texture.height(),
            depth_or_array_layers: texture.layers(),
        };
        let label = config.label.map(|s| s.to_owned());
        let tex_descriptor = TextureDescriptor {
            label: config.label,
            size,
            mip_level_count: config.mip_count.unwrap_or(1),
            sample_count: 1,
            dimension: T::wgpu_texture_dimension(),
            format: L::wgpu_texture_format(),
            usage: config.usage.to_wgpu_texture_usage(),
        };

        let wgpu_texture = framework.device.create_texture(&tex_descriptor);
        let binding_infos = texture.create_binding_info(&wgpu_texture, &framework.device);

        let gpu_texture = GpuTexture {
            phant_data: PhantomData,
            label,
            wgpu_texture,
            size: Extent3d {
                width: texture.width(),
                height: texture.height(),
                depth_or_array_layers: texture.layers(),
            },
            usage: config.usage,
            mip_count: config.mip_count,
            binding_infos,
        };
        if let Some(data) = texture.data() {
            gpu_texture.write_data(data, framework);
        }
        gpu_texture
    }

    pub(crate) fn height(&self) -> u32 {
        self.size.height
    }

    pub(crate) fn width(&self) -> u32 {
        self.size.width
    }
    pub(crate) fn layers(&self) -> u32 {
        self.size.depth_or_array_layers
    }

    pub(crate) fn texture(&self) -> &wgpu::Texture {
        &self.wgpu_texture
    }

    pub(crate) fn texture_view(&self, index: usize) -> &wgpu::TextureView {
        &self
            .binding_infos
            .get(index)
            .unwrap_or_else(|| panic!("This Texture doesn't have a TextureView at index {}", index))
            .view
    }
    pub(crate) fn sampler(&self, index: usize) -> &wgpu::Sampler {
        &self
            .binding_infos
            .get(index)
            .unwrap_or_else(|| panic!("This Texture doesn't have a Sampler at index {}", index))
            .sampler
    }

    pub(crate) fn bind_group(&self, index: usize) -> &wgpu::BindGroup {
        &self
            .binding_infos
            .get(index)
            .unwrap_or_else(|| panic!("This Texture doesn't have a BindGroupg at index {}", index))
            .bind_group
    }

    fn convert_region_y_to_wgpu_y(&self, y: u32, region_height: u32) -> u32 {
        self.height() - y - region_height
    }

    pub(crate) fn sample(
        &self,
        point: T::SamplingPointType,
        framework: &Framework,
    ) -> Result<L, TexelConversionError> {
        let texture_region = wgpu::ImageCopyTexture {
            texture: self.texture(),
            mip_level: 0,
            origin: point.origin(),
            aspect: wgpu::TextureAspect::All,
        };
        let oneshot_buffer = framework.buffer_oneshot(crate::BufferConfiguration::<u8> {
            initial_setup: crate::buffer::BufferInitialSetup::Size(
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
                    label: Some("Gpu -> Texel"),
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
        let color_bytes = oneshot_buffer.read_region(framework, (0, 4));
        L::from_bytes(&color_bytes)
    }

    pub(crate) fn write_data(&self, texels: &[L], framework: &Framework) {
        self.write_region(
            texels,
            T::SamplingPointType::from_wgpu_origin(Origin3d::ZERO),
            T::SamplingExtentsType::from_wgpu_extents(self.size.clone()),
            framework,
        );
    }

    pub(crate) fn write_region(
        &self,
        texels: &[L],
        region_origin: T::SamplingPointType,
        region_extents: T::SamplingExtentsType,
        framework: &Framework,
    ) {
        let wgpu_origin = region_origin.origin();
        let wgpu_extents = region_extents.extents();
        let total_size_to_copy =
            wgpu_extents.width * wgpu_extents.height * L::channel_count() * L::channel_size_bytes();
        let buffer_offset =
            wgpu_origin.x * wgpu_origin.y * L::channel_count() * L::channel_size_bytes();
        let region_bytes = bytemuck::cast_slice(texels);
        assert!(total_size_to_copy as usize <= region_bytes.len());

        let texture_region = wgpu::ImageCopyTexture {
            texture: &self.texture(),
            mip_level: 0,
            origin: wgpu_origin,
            aspect: wgpu::TextureAspect::All,
        };

        framework.queue.write_texture(
            texture_region,
            &region_bytes,
            wgpu::ImageDataLayout {
                offset: buffer_offset as u64,
                bytes_per_row: NonZeroU32::new(
                    self.width() * L::channel_count() * L::channel_size_bytes(),
                ),
                rows_per_image: NonZeroU32::new(self.height()),
            },
            wgpu_extents,
        );
    }

    pub(crate) fn read_data(&self, framework: &Framework) -> Result<T, TexelConversionError> {
        self.read_region(
            T::SamplingPointType::from_wgpu_origin(Origin3d::ZERO),
            T::SamplingExtentsType::from_wgpu_extents(self.size.clone()),
            framework,
        )
    }

    pub(crate) fn read_region(
        &self,
        origin: T::SamplingPointType,
        extents: T::SamplingExtentsType,
        framework: &Framework,
    ) -> Result<T, TexelConversionError> {
        let mut encoder =
            framework
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("copy texture to buffer"),
                });

        let mut wgpu_origin = origin.origin();
        let wgpu_extents = extents.extents();

        // Needed because textures in wgpu go from bottom to top, and we
        // pass coords from top to bottom
        wgpu_origin.y = self.convert_region_y_to_wgpu_y(wgpu_origin.y, wgpu_extents.height);

        let unpadded_width = wgpu_extents.width * L::channel_count() * L::channel_size_bytes();
        let pad_bytes = (wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
            - (unpadded_width % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT))
            % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_width = unpadded_width + pad_bytes;
        let oneshot_buffer = framework.buffer_oneshot(crate::BufferConfiguration {
            initial_setup: crate::buffer::BufferInitialSetup::Size::<u8>(
                (padded_width * wgpu_extents.height) as u64,
            ),
            buffer_type: crate::BufferType::Oneshot,
            allow_write: true,
            allow_read: true,
        });
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.texture(),
                mip_level: 0,
                origin: wgpu_origin,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &oneshot_buffer.inner_buffer(),
                layout: ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(padded_width),
                    rows_per_image: std::num::NonZeroU32::new(wgpu_extents.height),
                },
            },
            wgpu_extents,
        );
        framework.queue.submit(std::iter::once(encoder.finish()));

        let mut bytes = oneshot_buffer.read_all_sync(framework);
        if padded_width != unpadded_width {
            bytes = correct_bytes_for_padding::<L>(bytes, padded_width, wgpu_extents);
        }
        T::from_bytes(&bytes, extents)
    }

    pub(crate) fn clone_subregion(
        &self,
        origin: T::SamplingPointType,
        extents: T::SamplingExtentsType,
        framework: &Framework,
    ) -> GpuTexture<L, T> {
        let mut encoder =
            framework
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("copy texture to buffer"),
                });

        let wgpu_extents = extents.extents();
        let mut origin = origin.origin();
        origin.y = self.convert_region_y_to_wgpu_y(origin.y, wgpu_extents.height);
        // Needed because textures in wgpu go from bottom to top, and we
        // pass coords from top to bottom

        let new_texture = GpuTexture::new(
            T::empty(extents),
            TextureConfiguration {
                label: self.label.clone().map(|label| label + " clone").as_deref(),
                usage: self.usage.clone(),
                mip_count: self.mip_count.clone(),
            },
            framework,
        );

        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture(),
                mip_level: 0,
                origin,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyTexture {
                texture: &new_texture.texture(),
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu_extents,
        );
        framework.queue.submit(std::iter::once(encoder.finish()));
        new_texture
    }
}

fn correct_bytes_for_padding<L: Texel>(
    bytes: Vec<u8>,
    padded_width: u32,
    wgpu_extents: Extent3d,
) -> Vec<u8> {
    let padded_rows = bytes.chunks((padded_width) as usize);
    let unpadded_rows = padded_rows.into_iter().map(|c| {
        c.chunks((wgpu_extents.width * L::channel_count() * L::channel_size_bytes()) as usize)
    });
    unpadded_rows.fold(vec![], |vec, mut c| {
        let row_bytes = c.next().unwrap().to_owned();
        [vec, row_bytes].concat()
    })
}
