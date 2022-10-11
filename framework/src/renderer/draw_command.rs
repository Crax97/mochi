use crate::{
    framework::{BufferId, ShaderId, TextureId},
    Transform2d,
};

#[derive(Default)]
pub enum PrimitiveType {
    #[default]
    Noop,
    Texture2D {
        texture_id: TextureId,
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

pub struct DrawCommand {
    pub primitives: PrimitiveType,
    pub primitive_count: u32,
    pub instance_buffer_id: Option<BufferId>,
    pub additional_data: OptionalDrawData,
}
