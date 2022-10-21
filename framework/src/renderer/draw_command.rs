use crate::{
    framework::{BufferId, ShaderId, TextureId},
    Transform2d,
};

pub enum DrawMode {
    // The shader used supports instancing: all the instance data passed in the draw call
    // will be stored in an instance buffer
    Instanced,

    // The instances will be drawn in separated render passes
    Single,
}

#[derive(Default)]
pub enum PrimitiveType {
    #[default]
    Noop,
    Texture2D {
        texture_id: TextureId,
        instances: Vec<Transform2d>,
        flip_uv_y: bool,
        multiply_color: wgpu::Color,
    },
}

pub enum BindableResource {
    UniformBuffer(BufferId),
    Texture(TextureId),
}

#[derive(Default)]
pub struct OptionalDrawData {
    pub additional_vertex_buffers: Vec<BufferId>,
    pub additional_bindable_resource: Vec<BindableResource>,

    // If none, an appropriate shader will be picked by the renderer based on the draw_type
    pub shader: Option<ShaderId>,
}

impl OptionalDrawData {
    pub fn just_shader(shader: Option<ShaderId>) -> Self {
        Self {
            shader,
            ..Default::default()
        }
    }
}

pub struct DrawCommand {
    pub primitives: PrimitiveType,
    pub draw_mode: DrawMode,
    pub additional_data: OptionalDrawData,
}
