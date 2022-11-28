use framework::framework::{BufferId, ShaderId, TextureId};
use framework::shader::{BindElement, ShaderCreationInfo};
use framework::BufferConfiguration;
use framework::{Buffer, Framework};
use image_editor::layers::{LayerId, LayerType};
use wgpu::{BlendComponent, ShaderModuleDescriptor, ShaderSource};

use crate::tools::{EditorCommand, EditorContext};
use crate::{StrokeContext, StrokePath};

use super::stamp_operation::StampOperation;
use super::BrushEngine;

struct LayerReplaceCommand {
    old_layer_texture_id: TextureId,
    modified_layer: LayerId,
}
impl LayerReplaceCommand {
    pub fn new(modified_layer: LayerId, old_layer_texture_id: TextureId) -> Self {
        Self {
            old_layer_texture_id,
            modified_layer,
        }
    }
}

impl EditorCommand for LayerReplaceCommand {
    fn undo(&self, context: &mut EditorContext) -> Box<dyn EditorCommand> {
        let new_texture_id = match &context
            .image_editor
            .document()
            .get_layer(&self.modified_layer)
            .layer_type
        {
            LayerType::Image { texture, .. } => texture.clone(),
            LayerType::Chonky(..) => todo!(),
            LayerType::Group => unreachable!(),
        };
        context
            .image_editor
            .mutate_current_layer(|lay| lay.replace_texture(self.old_layer_texture_id.clone()));
        Box::new(LayerReplaceCommand::new(
            self.modified_layer,
            new_texture_id.clone(),
        ))
    }
}

pub struct Stamp {
    pub(crate) brush_texture: TextureId,
}

pub struct StampCreationInfo<'framework> {
    pub camera_buffer: &'framework Buffer,
}

impl Stamp {
    pub fn new(brush_texture: TextureId) -> Self {
        Self { brush_texture }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StampConfiguration {
    pub color_srgb: [u8; 3],
    pub opacity: u8,
    pub flow: f32,
    pub softness: f32,
    pub padding: [f32; 3],
    pub is_eraser: bool,
}

impl StampConfiguration {
    fn wgpu_color(&self) -> wgpu::Color {
        wgpu::Color {
            r: self.color_srgb[0] as f64 / 255.0,
            g: self.color_srgb[1] as f64 / 255.0,
            b: self.color_srgb[2] as f64 / 255.0,
            a: self.opacity as f64 / 255.0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BrushUniformData {
    pub softness: f32,
    pub padding: [f32; 3],
}

impl From<StampConfiguration> for BrushUniformData {
    fn from(cfg: StampConfiguration) -> Self {
        Self {
            softness: cfg.softness,
            padding: cfg.padding,
        }
    }
}

pub struct StrokingEngine {
    current_stamp: usize,
    stamps: Vec<Stamp>,
    stamp_configuration: StampConfiguration,
    wants_update_brush_settings: bool,
    brush_shader_id: ShaderId,
    eraser_shader_id: ShaderId,
    brush_settings_buffer_id: BufferId,
}

impl StrokingEngine {
    pub fn new(initial_stamp: Stamp, framework: &mut Framework) -> Self {
        let brush_fragment = framework
            .shader_compiler
            .compile(include_str!("brush_fragment.wgsl"));
        let brush_shader_info = ShaderCreationInfo::using_default_vertex_instanced(
            ShaderModuleDescriptor {
                label: Some("Brush shader"),
                source: ShaderSource::Naga(brush_fragment),
            },
            framework,
        )
        .with_bind_element(BindElement::Texture) // 2: texture + sampler
        .with_bind_element(BindElement::UniformBuffer); // 3: brush settings

        let eraser_blend_state = wgpu::BlendState {
            color: BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        };

        let brush_fragment = framework
            .shader_compiler
            .compile(include_str!("brush_fragment.wgsl"));
        let eraser_shader_info = ShaderCreationInfo::using_default_vertex_instanced(
            ShaderModuleDescriptor {
                label: Some("Eraser shader"),
                source: ShaderSource::Naga(brush_fragment),
            },
            framework,
        )
        .with_bind_element(BindElement::Texture) // 2: texture + sampler
        .with_bind_element(BindElement::UniformBuffer) // 3: brush settings
        .with_blend_state(eraser_blend_state);

        let stamp_config = StampConfiguration {
            color_srgb: [0, 0, 0],
            opacity: 255,
            flow: 1.0,
            softness: 0.2,
            padding: [0.0; 3],
            is_eraser: false,
        };

        let brush_shader_id = framework.create_shader(brush_shader_info);
        let eraser_shader_id = framework.create_shader(eraser_shader_info);
        let brush_settings_buffer_id =
            framework.allocate_typed_buffer(BufferConfiguration::<BrushUniformData> {
                initial_setup: framework::buffer::BufferInitialSetup::Data(&vec![
                    BrushUniformData::from(stamp_config.clone()),
                ]),
                buffer_type: framework::BufferType::Uniform,
                gpu_copy_dest: true,
                gpu_copy_source: false,
                cpu_copy_dest: false,
                cpu_copy_source: false,
            });

        Self {
            stamps: vec![initial_stamp],
            current_stamp: 0,
            stamp_configuration: stamp_config,
            wants_update_brush_settings: true,
            brush_shader_id,
            brush_settings_buffer_id,
            eraser_shader_id,
        }
    }

    pub fn create_stamp(&self, brush_texture: TextureId) -> Stamp {
        Stamp::new(brush_texture)
    }

    pub fn settings(&self) -> StampConfiguration {
        self.stamp_configuration.clone()
    }

    pub fn set_new_settings(&mut self, settings: StampConfiguration) {
        self.stamp_configuration = settings;
        self.wants_update_brush_settings = true; // Defer updating brush settings until stroke
    }

    fn current_stamp(&self) -> &Stamp {
        self.stamps
            .get(self.current_stamp)
            .expect("Could not find the given index in stamp array")
    }

    fn create_clone_of_current_layer_texture(
        context: &mut EditorContext,
    ) -> (TextureId, TextureId) {
        let modified_layer = context
            .image_editor
            .document()
            .current_layer_index()
            .unwrap()
            .clone();
        let layer = context.image_editor.document().get_layer(&modified_layer);
        let old_layer_texture_id = match &layer.layer_type {
            LayerType::Image { texture, .. } => texture.clone(),
            _ => unreachable!(),
        };
        let (width, height) = context
            .framework
            .texture2d_dimensions(&old_layer_texture_id);

        let new_texture_id =
            context
                .framework
                .texture2d_copy_subregion(&old_layer_texture_id, 0, 0, width, height);

        (old_layer_texture_id, new_texture_id)
    }

    pub fn toggle_eraser(&mut self) {
        self.stamp_configuration.is_eraser = !self.stamp_configuration.is_eraser;
    }

    fn update_brush_settings(&self, framework: &mut Framework) {
        framework.buffer_write_sync(
            &self.brush_settings_buffer_id,
            vec![BrushUniformData::from(self.stamp_configuration)],
        );
    }
}

impl BrushEngine for StrokingEngine {
    fn stroke(
        &mut self,
        path: StrokePath,
        context: StrokeContext,
    ) -> Option<Box<dyn EditorCommand>> {
        if self.wants_update_brush_settings {
            self.update_brush_settings(context.framework);
            self.wants_update_brush_settings = false;
        }

        let StrokeContext {
            framework,
            editor,
            renderer,
        } = context;
        let path_bounds = path.bounds();
        editor.mutate_current_layer(move |layer| {
            layer.execute_operation(
                StampOperation {
                    path,
                    brush: self.current_stamp().brush_texture.clone(),
                    color: self.settings().wgpu_color(),
                    is_eraser: self.settings().is_eraser,
                    brush_settings_buffer: self.brush_settings_buffer_id.clone(),
                    eraser_shader_id: self.eraser_shader_id.clone(),
                    brush_shader_id: self.brush_shader_id.clone(),
                },
                path_bounds,
                renderer,
                framework,
            )
        });

        None
    }

    fn begin_stroking(&mut self, context: &mut EditorContext) -> Option<Box<dyn EditorCommand>> {
        match &context.image_editor.document().current_layer().layer_type {
            LayerType::Image { .. } => {
                let (old_layer_texture_id, new_texture_id) =
                    StrokingEngine::create_clone_of_current_layer_texture(context);
                context
                    .image_editor
                    .mutate_current_layer(|lay| match &mut lay.layer_type {
                        LayerType::Image { .. } => lay.replace_texture(new_texture_id.clone()),
                        _ => unreachable!(),
                    });
                let cmd = LayerReplaceCommand::new(
                    context
                        .image_editor
                        .document()
                        .current_layer_index()
                        .unwrap()
                        .clone(),
                    old_layer_texture_id,
                );
                Some(Box::new(cmd))
            }
            LayerType::Chonky(_) => None,
            LayerType::Group => unreachable!(),
        }
    }
}
