use wgpu::{BindGroup, Sampler, TextureAspect, TextureFormat, TextureSampleType, TextureView};

use super::texture::TexelConversionError;

pub struct BindingInfo {
    pub(crate) view: TextureView,
    pub(crate) sampler: Sampler,
    pub(crate) bind_group: BindGroup,
}

pub enum ChannelType {
    U8,
    F24,
    F32,
}

impl ChannelType {
    fn size_bytes(&self) -> usize {
        match self {
            ChannelType::U8 => 1,
            ChannelType::F24 => 3,
            ChannelType::F32 => 4,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AspectInfo {
    pub aspect: TextureAspect,
    pub format: TextureFormat,
    pub sample_type: TextureSampleType,
}

pub trait Texel: bytemuck::Pod + bytemuck::Zeroable {
    fn from_bytes(bytes: &[u8]) -> Result<Self, TexelConversionError>
    where
        Self: Sized;

    fn channels() -> &'static [ChannelType];
    fn channel_count() -> usize {
        Self::channels().len()
    }
    fn total_texel_size_bytes() -> usize {
        Self::channels()
            .iter()
            .fold(0usize, |acc, curr| acc + curr.size_bytes())
    }
    fn wgpu_texture_format() -> TextureFormat;
    fn supported_aspects() -> &'static [AspectInfo];

    fn wgpu_color(&self) -> wgpu::Color;
    fn bytes(&self) -> &[u8];
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct RgbaU8([u8; 4]);

impl Texel for RgbaU8 {
    fn from_bytes(bytes: &[u8]) -> Result<Self, TexelConversionError> {
        if bytes.len() < Self::total_texel_size_bytes() as usize {
            return Err(TexelConversionError::NotEnoughData);
        }

        Ok(RgbaU8([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn channels() -> &'static [ChannelType] {
        static RGBA_CHANNELS: &[ChannelType] = &[
            ChannelType::U8,
            ChannelType::U8,
            ChannelType::U8,
            ChannelType::U8,
        ];
        RGBA_CHANNELS
    }

    fn wgpu_texture_format() -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba8UnormSrgb
    }

    fn supported_aspects() -> &'static [AspectInfo] {
        static ASPECTS: &[AspectInfo] = &[AspectInfo {
            aspect: TextureAspect::All,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            sample_type: TextureSampleType::Float { filterable: true },
        }];
        ASPECTS
    }

    fn wgpu_color(&self) -> wgpu::Color {
        wgpu::Color {
            r: self.0[0] as f64 / 255.0,
            g: self.0[1] as f64 / 255.0,
            b: self.0[2] as f64 / 255.0,
            a: self.0[3] as f64 / 255.0,
        }
    }

    fn bytes(&self) -> &[u8] {
        &self.0
    }
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct DepthStencilTexel([u8; 4]);

impl Texel for DepthStencilTexel {
    fn from_bytes(bytes: &[u8]) -> Result<Self, TexelConversionError>
    where
        Self: Sized,
    {
        if bytes.len() < Self::total_texel_size_bytes() as usize {
            return Err(TexelConversionError::NotEnoughData);
        }
        Ok(DepthStencilTexel([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn channels() -> &'static [ChannelType] {
        static DEPTH_STENCIL_CHANNELS: &[ChannelType] = &[ChannelType::F32, ChannelType::U8];
        DEPTH_STENCIL_CHANNELS
    }

    fn wgpu_texture_format() -> TextureFormat {
        TextureFormat::Depth24PlusStencil8
    }
    fn supported_aspects() -> &'static [AspectInfo] {
        static ASPECTS: &[AspectInfo] = &[
            AspectInfo {
                aspect: TextureAspect::DepthOnly,
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                sample_type: TextureSampleType::Depth,
            },
            AspectInfo {
                aspect: TextureAspect::StencilOnly,
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                sample_type: TextureSampleType::Uint,
            },
        ];
        ASPECTS
    }

    fn wgpu_color(&self) -> wgpu::Color {
        wgpu::Color {
            r: self.0[0] as f64 / 255.0,
            g: self.0[1] as f64 / 255.0,
            b: self.0[2] as f64 / 255.0,
            a: self.0[3] as f64 / 255.0,
        }
    }

    fn bytes(&self) -> &[u8] {
        &self.0
    }
}

impl DepthStencilTexel {
    pub fn depth(&self) -> f32 {
        const MAX_DEPTH: u32 = 2u32.pow(24);
        let depth = u32::from_le_bytes([self.0[0], self.0[1], self.0[2], 0]) as f32;
        depth / MAX_DEPTH as f32
    }
    pub fn stencil(&self) -> u8 {
        self.0[3]
    }
}
