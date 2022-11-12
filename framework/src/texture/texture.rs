use std::num::NonZeroU8;

use wgpu::{Extent3d, Origin3d, TextureDimension};

use crate::{BindingInfo, Texel};

pub trait SamplingOrigin {
    fn origin(&self) -> Origin3d;
}
pub trait SamplingExtents {
    fn extents(&self) -> Extent3d;
}

impl SamplingOrigin for (u32, u32) {
    fn origin(&self) -> Origin3d {
        Origin3d {
            x: self.0,
            y: self.1,
            z: 0,
        }
    }
}
impl SamplingOrigin for (u32, u32, u32) {
    fn origin(&self) -> Origin3d {
        Origin3d {
            x: self.0,
            y: self.1,
            z: self.2,
        }
    }
}

impl SamplingExtents for (u32, u32) {
    fn extents(&self) -> Extent3d {
        Extent3d {
            width: self.0,
            height: self.1,
            depth_or_array_layers: 0,
        }
    }
}
impl SamplingExtents for (u32, u32, u32) {
    fn extents(&self) -> Extent3d {
        Extent3d {
            width: self.0,
            height: self.1,
            depth_or_array_layers: self.2,
        }
    }
}

#[derive(Debug)]
pub enum TexelConversionError {
    NotEnoughData,
}

pub trait Texture<T: Texel> {
    type SamplingPointType: SamplingOrigin;
    type SamplingExtentsType: SamplingExtents;
    fn wgpu_texture_dimension() -> TextureDimension;
    fn from_texels(
        texels: Vec<T>,
        size: Self::SamplingExtentsType,
    ) -> Result<Self, TexelConversionError>
    where
        Self: Sized;
    fn empty(size: Self::SamplingExtentsType) -> Self;
    fn data(&self) -> Option<&[T]>;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn layers(&self) -> u32;
    fn create_binding_info(
        &self,
        texture: &wgpu::Texture,
        device: &wgpu::Device,
    ) -> Vec<BindingInfo>;
}

pub struct Texture2D<T: Texel> {
    data: Option<Vec<T>>,
    width: u32,
    height: u32,
}

impl<T: Texel> Texture2D<T> {}

impl<T: Texel> Texture<T> for Texture2D<T> {
    type SamplingPointType = (u32, u32);
    type SamplingExtentsType = (u32, u32);
    fn wgpu_texture_dimension() -> wgpu::TextureDimension {
        wgpu::TextureDimension::D2
    }

    fn create_binding_info(
        &self,
        texture: &wgpu::Texture,
        device: &wgpu::Device,
    ) -> Vec<BindingInfo> {
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Framework Texture view"),
            format: Some(T::wgpu_texture_format()),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
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
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("RgbaU8 Bind Group layout"),
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
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("RgbaU8 Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        vec![BindingInfo {
            view,
            sampler,
            bind_group,
        }]
    }

    fn data(&self) -> Option<&[T]> {
        self.data.as_deref()
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn layers(&self) -> u32 {
        1
    }

    fn from_texels(
        texels: Vec<T>,
        size: Self::SamplingExtentsType,
    ) -> Result<Self, TexelConversionError>
    where
        Self: Sized,
    {
        if texels.len() < (size.0 * size.1) as usize {
            return Err(TexelConversionError::NotEnoughData);
        }

        let extents = size.extents();
        Ok(Self {
            data: Some(texels),
            width: extents.width,
            height: extents.height,
        })
    }
    fn empty(size: Self::SamplingExtentsType) -> Self {
        let extents = size.extents();
        Self {
            data: None,
            width: extents.width,
            height: extents.height,
        }
    }
}
