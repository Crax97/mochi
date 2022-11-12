mod gpu_texture;
mod texels;
mod texture;

pub use gpu_texture::*;
pub use texels::*;
pub use texture::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct TextureUsage {
    pub cpu_write: bool,
    pub cpu_read: bool,
    pub use_as_render_target: bool,
}

#[derive(Debug, Default)]
pub struct TextureConfiguration<'a> {
    pub label: Option<&'a str>,
    pub usage: TextureUsage,
    pub mip_count: Option<u32>,
}

impl TextureUsage {
    const RWRT: TextureUsage = TextureUsage {
        cpu_write: true,
        cpu_read: true,
        use_as_render_target: true,
    };
    const READ_WRITE: TextureUsage = TextureUsage {
        cpu_write: true,
        cpu_read: true,
        use_as_render_target: false,
    };
    const READ_ONLY: TextureUsage = TextureUsage {
        cpu_write: false,
        cpu_read: true,
        use_as_render_target: false,
    };
    const WRITE_ONLY: TextureUsage = TextureUsage {
        cpu_write: true,
        cpu_read: false,
        use_as_render_target: false,
    };

    pub(crate) fn to_wgpu_texture_usage(self) -> wgpu::TextureUsages {
        let check = |enable, flag| {
            if enable {
                flag
            } else {
                wgpu::TextureUsages::empty()
            }
        };
        check(self.cpu_write, wgpu::TextureUsages::COPY_DST)
            | check(self.cpu_read, wgpu::TextureUsages::COPY_SRC)
            | check(
                self.use_as_render_target,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            )
            | wgpu::TextureUsages::TEXTURE_BINDING
    }
}

pub type RgbaTexture2D = Texture2D<RgbaU8>;
pub type GpuRgbaTexture2D = GpuTexture<RgbaU8, RgbaTexture2D>;

impl GpuRgbaTexture2D {}
