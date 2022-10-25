use cgmath::{point3, vec2, Rad, SquareMatrix, Transform};
use framework::framework::{BufferId, ShaderId, TextureId};
use framework::renderer::draw_command::{
    BindableResource, DrawCommand, DrawMode, OptionalDrawData, PrimitiveType,
};
use framework::shader::{BindElement, ShaderCreationInfo};
use framework::{Buffer, Framework};
use framework::{BufferConfiguration, Transform2d};
use image_editor::layers::{BitmapLayer, LayerIndex, LayerType};
use wgpu::{BlendComponent, ShaderModuleDescriptor, ShaderSource};

use crate::tools::{EditorCommand, EditorContext};
use crate::{StrokeContext, StrokePath};

use super::BrushEngine;

struct LayerReplaceCommand {
    old_layer_texture_id: TextureId,
    modified_layer: LayerIndex,
}
impl LayerReplaceCommand {
    pub fn new(modified_layer: LayerIndex, old_layer_texture_id: TextureId) -> Self {
        Self {
            old_layer_texture_id,
            modified_layer,
        }
    }
}

impl EditorCommand for LayerReplaceCommand {
    fn undo(&self, context: &mut EditorContext) -> Box<dyn EditorCommand> {
        let new_texture_id = context
            .image_editor
            .document()
            .get_layer(&self.modified_layer)
            .bitmap
            .texture()
            .clone();
        context.image_editor.mutate_document(|doc| {
            doc.mutate_layer(&self.modified_layer, |lay| match &mut lay.layer_type {
                LayerType::Bitmap => {
                    lay.replace_texture(self.old_layer_texture_id.clone());
                }
            })
        });
        Box::new(LayerReplaceCommand::new(
            self.modified_layer,
            new_texture_id.clone(),
        ))
    }
}

pub struct Stamp {
    pub(crate) brush_texture: BitmapLayer,
}

pub struct StampCreationInfo<'framework> {
    pub camera_buffer: &'framework Buffer,
}

impl Stamp {
    pub fn new(brush_texture: BitmapLayer) -> Self {
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
    pub fn new(initial_stamp: Stamp, framework: &Framework) -> Self {
        let brush_fragment = framework
            .shader_compiler
            .compile(include_str!("brush_fragment.wgsl"))
            .unwrap();
        let brush_shader_info = ShaderCreationInfo::using_default_vertex_instanced(
            framework,
            ShaderModuleDescriptor {
                label: Some("Brush shader"),
                source: ShaderSource::Naga(brush_fragment),
            },
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
            .compile(include_str!("brush_fragment.wgsl"))
            .unwrap();
        let eraser_shader_info = ShaderCreationInfo::using_default_vertex_instanced(
            framework,
            ShaderModuleDescriptor {
                label: Some("Eraser shader"),
                source: ShaderSource::Naga(brush_fragment),
            },
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
                allow_write: true,
                allow_read: false,
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

    pub fn create_stamp(&self, brush_texture: BitmapLayer) -> Stamp {
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
        let modified_layer = context.image_editor.document().current_layer_index();
        let layer = context.image_editor.document().get_layer(&modified_layer);
        let old_layer_texture_id = layer.bitmap.texture().clone();
        let (width, height) = context
            .image_editor
            .framework()
            .texture2d_dimensions(&old_layer_texture_id);

        let new_texture_id = context.image_editor.framework().texture2d_copy_subregion(
            &old_layer_texture_id,
            0,
            0,
            width,
            height,
        );

        (old_layer_texture_id, new_texture_id)
    }

    pub fn toggle_eraser(&mut self) {
        self.stamp_configuration.is_eraser = !self.stamp_configuration.is_eraser;
    }

    fn update_brush_settings(&self, framework: &Framework) {
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
            self.update_brush_settings(context.editor.framework());
            self.wants_update_brush_settings = false;
        }

        let layer = context.editor.document().current_layer();
        match layer.layer_type {
            // TODO: Deal with difference between current_layer and buffer_layer size
            LayerType::Bitmap => {
                // 1. Create draw info

                let current_layer_transform = layer.transform();
                let inv_layer_matrix = current_layer_transform.matrix().invert();
                if let Some(inv_layer_matrix) = inv_layer_matrix {
                    let transforms: Vec<Transform2d> = path
                        .points
                        .iter()
                        .map(|pt| {
                            let origin_inv = inv_layer_matrix.transform_point(point3(
                                pt.position.x,
                                pt.position.y,
                                0.0,
                            ));
                            Transform2d {
                                position: origin_inv,
                                scale: vec2(pt.size, pt.size),
                                rotation_radians: Rad(0.0),
                            }
                        })
                        .collect();

                    // 2. Do draw

                    let stamp = self.current_stamp().brush_texture.texture();
                    context.renderer.begin(&layer.bitmap.camera(), None);
                    context.renderer.set_viewport(Some((
                        0.0,
                        0.0,
                        layer.bitmap.size().x,
                        layer.bitmap.size().y,
                    )));
                    context.renderer.draw(DrawCommand {
                        primitives: PrimitiveType::Texture2D {
                            texture_id: stamp.clone(),
                            instances: transforms,
                            flip_uv_y: true,
                            multiply_color: self.settings().wgpu_color(),
                        },
                        draw_mode: DrawMode::Instanced,
                        additional_data: OptionalDrawData {
                            shader: Some(if self.settings().is_eraser {
                                self.eraser_shader_id.clone()
                            } else {
                                self.brush_shader_id.clone()
                            }),
                            additional_bindable_resource: vec![BindableResource::UniformBuffer(
                                self.brush_settings_buffer_id.clone(),
                            )],
                            ..Default::default()
                        },
                    });
                    context
                        .renderer
                        .end_on_texture(layer.bitmap.texture(), None);
                }
            }
        }
        None
    }

    fn begin_stroking(&mut self, context: &mut EditorContext) -> Option<Box<dyn EditorCommand>> {
        let modified_layer = context.image_editor.document().current_layer_index();
        let (old_layer_texture_id, new_texture_id) =
            StrokingEngine::create_clone_of_current_layer_texture(context);
        context.image_editor.mutate_document(|doc| {
            doc.mutate_layer(&modified_layer, |lay| match &mut lay.layer_type {
                LayerType::Bitmap => lay.replace_texture(new_texture_id.clone()),
            })
        });
        let cmd = LayerReplaceCommand::new(
            context.image_editor.document().current_layer_index(),
            old_layer_texture_id,
        );
        Some(Box::new(cmd))
    }
}
