use crate::framework::{DepthStencilTextureId, TextureId};

#[derive(Clone, Debug)]
struct Framebuffer {
    pub(crate) color_textures_id: Vec<TextureId>,
    pub(crate) depth_stencil_texture_id: Option<DepthStencilTextureId>,
}

struct FramebufferBuilder {
    inner_framebuffer: Framebuffer,
}

impl FramebufferBuilder {
    pub fn new() -> Self {
        Self {
            inner_framebuffer: Framebuffer {
                color_textures_id: vec![],
                depth_stencil_texture_id: None,
            },
        }
    }

    pub fn add_color_target(mut self, color_texture: TextureId) -> Self {
        self.inner_framebuffer.color_textures_id.push(color_texture);
        self
    }

    pub fn with_depth_target(mut self, depth_stencil_texture_id: DepthStencilTextureId) -> Self {
        self.inner_framebuffer.depth_stencil_texture_id = Some(depth_stencil_texture_id);
        self
    }
    pub fn build(self) -> Framebuffer {
        self.inner_framebuffer
    }
}
