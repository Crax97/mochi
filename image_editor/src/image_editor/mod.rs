mod ab_render_target;
pub mod blend_settings;
pub mod document;
pub mod image_editor;
pub mod image_editor_event;
pub mod layers;
pub mod selection;

use framework::framework::ShaderId;
use framework::shader::BindElement;
use framework::shader::ShaderCreationInfo;
use framework::Framework;
pub use image_editor::ImageEditor;
pub use image_editor::LayerConstructionInfo;
pub use image_editor_event::ImageEditorEvent;
use once_cell::sync::OnceCell;
use wgpu::DepthBiasState;
use wgpu::DepthStencilState;
use wgpu::StencilFaceState;
use wgpu::StencilState;

#[derive(Debug)]
pub struct ImageEditorGlobals {
    pub draw_on_stencil_buffer_shader_id: ShaderId,
    pub draw_masked_stencil_buffer_shader_id: ShaderId,
    pub draw_masked_inverted_stencil_buffer_shader_id: ShaderId,
    pub dotted_shader: ShaderId,
    pub blended_shader: ShaderId,
}

static INSTANCE: OnceCell<ImageEditorGlobals> = OnceCell::new();
fn make_globals(framework: &mut Framework) -> ImageEditorGlobals {
    let info = ShaderCreationInfo::using_default_vertex_fragment(framework);
    let draw_on_stencil_buffer_shader_id = framework.create_shader(info);
    let info = ShaderCreationInfo::using_default_vertex_fragment(framework);
    let draw_masked_stencil_buffer_shader_id = framework.create_shader(info);

    let info = ShaderCreationInfo::using_default_vertex_fragment(framework).with_depth_state(Some(
        DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            stencil: StencilState {
                front: StencilFaceState {
                    compare: wgpu::CompareFunction::NotEqual,
                    pass_op: wgpu::StencilOperation::Keep,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                },
                back: StencilFaceState {
                    compare: wgpu::CompareFunction::NotEqual,
                    pass_op: wgpu::StencilOperation::Keep,
                    fail_op: wgpu::StencilOperation::Keep,
                    depth_fail_op: wgpu::StencilOperation::Keep,
                },
                read_mask: 0xFFFFFFF,
                write_mask: 0xFFFFFFF,
            },
            bias: DepthBiasState::default(),
        },
    ));
    let draw_masked_inverted_stencil_buffer_shader_id = framework.create_shader(info);

    let dotted_module_descriptor = framework.shader_compiler.compile_into_shader_description(
        "Dotted shader",
        include_str!("shaders/dotted_selection.wgsl"),
    );
    let dotted_info = ShaderCreationInfo::using_default_vertex(dotted_module_descriptor, framework)
        .with_bind_element(BindElement::Texture); // 2: diffuse texture + sampler
    let dotted_shader = framework.create_shader(dotted_info);

    let blended_shader = framework.shader_compiler.compile_into_shader_description(
        "Layer draw shader",
        include_str!("layers/layer_fragment.wgsl"),
    );
    let blended_shader_info = ShaderCreationInfo::using_default_vertex(blended_shader, framework)
        .with_bind_element(BindElement::Texture) // Bottom layer
        .with_bind_element(BindElement::Texture) // Top layer
        .with_bind_element(BindElement::UniformBuffer); // Blend settings
    let blended_shader = framework.create_shader(blended_shader_info);

    ImageEditorGlobals {
        draw_on_stencil_buffer_shader_id,
        draw_masked_stencil_buffer_shader_id,
        draw_masked_inverted_stencil_buffer_shader_id,
        dotted_shader,
        blended_shader,
    }
}

pub(crate) fn init_globals(framework: &mut Framework) {
    if let None = INSTANCE.get() {
        INSTANCE.set(make_globals(framework)).unwrap();
    }
}

pub fn global_selection_data() -> &'static ImageEditorGlobals {
    INSTANCE.get().unwrap()
}
