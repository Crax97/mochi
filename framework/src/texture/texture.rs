use std::num::NonZeroU8;

use wgpu::{BindGroupLayout, Extent3d, Origin3d, TextureDimension};

use crate::{BindingInfo, Framework, Texel};

pub trait SamplingOrigin {
    fn origin(&self) -> Origin3d;
    fn from_wgpu_origin(origin: Origin3d) -> Self;
}
pub trait SamplingExtents {
    fn extents(&self) -> Extent3d;
    fn from_wgpu_extents(extents: Extent3d) -> Self;
    fn total_count(&self) -> u32;
}

impl SamplingOrigin for (u32, u32) {
    fn origin(&self) -> Origin3d {
        Origin3d {
            x: self.0,
            y: self.1,
            z: 0,
        }
    }
    fn from_wgpu_origin(origin: Origin3d) -> Self {
        (origin.x, origin.y)
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
    fn from_wgpu_origin(origin: Origin3d) -> Self {
        (origin.x, origin.y, origin.z)
    }
}

impl SamplingExtents for (u32, u32) {
    fn extents(&self) -> Extent3d {
        Extent3d {
            width: self.0,
            height: self.1,
            depth_or_array_layers: 1,
        }
    }

    fn from_wgpu_extents(extents: Extent3d) -> Self {
        (extents.width, extents.height)
    }

    fn total_count(&self) -> u32 {
        self.0 * self.1
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

    fn from_wgpu_extents(extents: Extent3d) -> Self {
        (extents.width, extents.height, extents.depth_or_array_layers)
    }

    fn total_count(&self) -> u32 {
        self.0 * self.1 * self.2
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
    fn from_repeated_texel(
        t: T,
        size: Self::SamplingExtentsType,
    ) -> Result<Self, TexelConversionError>
    where
        Self: Sized;
    fn from_texels(
        texels: Vec<T>,
        size: Self::SamplingExtentsType,
    ) -> Result<Self, TexelConversionError>
    where
        Self: Sized;
    fn from_bytes(
        bytes: &[u8],
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

pub fn texture2d_bind_group_layout(framework: &Framework) -> BindGroupLayout {
    framework
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Depth Texture Bindg layout"),
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
        })
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
        let aspects = T::supported_aspects();
        let mut binding_infos = vec![];
        for aspect in aspects {
            let view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(format!("Texture2D view aspect: {:?}", aspect).as_str()),
                format: Some(aspect.format),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: aspect.aspect,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });
            let sampler = if aspect.create_sampler {
                Some(device.create_sampler(&wgpu::SamplerDescriptor {
                    label: Some(format!("Texture2D sampler, aspect: {:?}", aspect).as_str()),
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
                }))
            } else {
                None
            };

            let bind_group = match &sampler {
                Some(sampler) => {
                    let layout =
                        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            label: Some(
                                format!("Texture2D BindGroup Layout, aspect: {:?}", aspect)
                                    .as_str(),
                            ),
                            entries: &[
                                wgpu::BindGroupLayoutEntry {
                                    binding: 0,
                                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                    ty: wgpu::BindingType::Texture {
                                        sample_type: aspect.sample_type,
                                        view_dimension: wgpu::TextureViewDimension::D2,
                                        multisampled: false,
                                    },
                                    count: None,
                                },
                                wgpu::BindGroupLayoutEntry {
                                    binding: 1,
                                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                    ty: wgpu::BindingType::Sampler(
                                        wgpu::SamplerBindingType::Filtering,
                                    ),
                                    count: None,
                                },
                            ],
                        });
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some(format!("Texture2D BindGroup, aspect: {:?}", aspect).as_str()),
                        layout: &layout,
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
                    })
                }
                None => {
                    let layout =
                        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            label: Some(
                                format!(
                                    "Texture2D BindGroup Layout, aspect: {:?} no sampler",
                                    aspect
                                )
                                .as_str(),
                            ),
                            entries: &[wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    sample_type: aspect.sample_type,
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    multisampled: false,
                                },
                                count: None,
                            }],
                        });

                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some(format!("Texture2D BindGroup, aspect: {:?}", aspect).as_str()),
                        layout: &layout,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view),
                        }],
                    })
                }
            };
            binding_infos.push(BindingInfo {
                view,
                sampler,
                bind_group,
            });
        }
        binding_infos
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

    fn from_bytes(
        bytes: &[u8],
        size: Self::SamplingExtentsType,
    ) -> Result<Self, TexelConversionError>
    where
        Self: Sized,
    {
        if bytes.len() < (size.0 * size.1) as usize * T::total_texel_size_bytes() {
            return Err(TexelConversionError::NotEnoughData);
        }
        let texels: &[T] = bytemuck::cast_slice(bytes);
        let texels = Vec::from_iter(texels.iter().map(|t| t.clone()));
        Self::from_texels(texels, size)
    }

    fn from_repeated_texel(
        t: T,
        size: Self::SamplingExtentsType,
    ) -> Result<Self, TexelConversionError>
    where
        Self: Sized,
    {
        let collected_texels = std::iter::repeat(t)
            .take(size.total_count() as usize)
            .collect();
        Self::from_texels(collected_texels, size)
    }
}
