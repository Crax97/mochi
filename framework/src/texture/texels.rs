use wgpu::{BindGroup, Sampler, TextureFormat, TextureView};

use super::texture::TexelConversionError;

pub struct BindingInfo {
    pub(crate) view: TextureView,
    pub(crate) sampler: Sampler,
    pub(crate) bind_group: BindGroup,
}

pub trait Texel: bytemuck::Pod + bytemuck::Zeroable {
    fn from_bytes(bytes: &[u8]) -> Result<Self, TexelConversionError>
    where
        Self: Sized;

    fn channel_count() -> u32;
    fn channel_size_bytes() -> u32;
    fn wgpu_texture_format() -> TextureFormat;

    fn wgpu_color(&self) -> wgpu::Color;
    fn bytes(&self) -> &[u8];
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct RgbaU8([u8; 4]);

impl Texel for RgbaU8 {
    fn from_bytes(bytes: &[u8]) -> Result<Self, TexelConversionError> {
        if bytes.len() < Self::channel_count() as usize {
            return Err(TexelConversionError::NotEnoughData);
        }

        Ok(RgbaU8([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn channel_count() -> u32 {
        4
    }

    fn channel_size_bytes() -> u32 {
        1
    }

    fn wgpu_texture_format() -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba8UnormSrgb
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
