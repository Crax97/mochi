use cgmath::{point2, point3, vec2, Rad, SquareMatrix, Transform};
use framework::framework::{BufferId, TextureId};
use framework::renderer::draw_command::{DrawCommand, DrawMode, PrimitiveType};
use framework::scene::{Camera2d, Camera2dUniformBlock};
use framework::{Buffer, Framework, MeshInstance2D};
use framework::{BufferConfiguration, Transform2d};
use image_editor::layers::{BitmapLayer, LayerIndex, LayerType};

use crate::tools::{EditorCommand, EditorContext};
use crate::{StrokeContext, StrokePath};

use super::BrushEngine;

struct LayerReplaceCommand {
    old_layer_texture_id: TextureId,
    modified_layer: LayerIndex,
}
impl LayerReplaceCommand {
    pub fn new(
        context: &mut EditorContext,
        modified_layer: LayerIndex,
        old_layer_texture_id: TextureId,
    ) -> Self {
        Self {
            old_layer_texture_id,
            modified_layer,
        }
    }
}

impl EditorCommand for LayerReplaceCommand {
    fn undo(&self, context: &mut EditorContext) -> Box<dyn EditorCommand> {
        let new_texture_id = match context
            .image_editor
            .document()
            .get_layer(&self.modified_layer)
            .layer_type
        {
            LayerType::Bitmap(ref bm) => bm.texture().clone(),
        };
        context.image_editor.mutate_document(|doc| {
            doc.mutate_layer(&self.modified_layer, |lay| match &mut lay.layer_type {
                LayerType::Bitmap(bm) => bm.replace_texture(self.old_layer_texture_id.clone()),
            })
        });
        Box::new(LayerReplaceCommand::new(
            context,
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
pub struct StampUniformData {
    pub color: [f32; 4],
    pub flow: f32,
    pub softness: f32,
    pub padding: [f32; 3],
}

impl From<StampConfiguration> for StampUniformData {
    fn from(cfg: StampConfiguration) -> Self {
        let color = [
            cfg.color_srgb[0] as f32 / 255.0,
            cfg.color_srgb[1] as f32 / 255.0,
            cfg.color_srgb[2] as f32 / 255.0,
            cfg.opacity as f32 / 255.0,
        ];
        Self {
            color,
            flow: cfg.flow,
            softness: cfg.softness,
            padding: cfg.padding,
        }
    }
}

pub struct StrokingEngine {
    current_stamp: usize,
    stamps: Vec<Stamp>,
    stamp_configuration: StampConfiguration,
}

impl StrokingEngine {
    pub fn new(initial_stamp: Stamp, framework: &Framework) -> Self {
        Self {
            stamps: vec![initial_stamp],
            current_stamp: 0,
            stamp_configuration: StampConfiguration {
                color_srgb: [0, 0, 0],
                opacity: 255,
                flow: 1.0,
                softness: 0.2,
                padding: [0.0; 3],
                is_eraser: false,
            },
        }
    }

    pub fn create_stamp(&self, brush_texture: BitmapLayer) -> Stamp {
        Stamp::new(brush_texture)
    }

    pub fn settings(&self) -> StampConfiguration {
        self.stamp_configuration.clone()
    }

    pub fn set_new_settings(&mut self, framework: &Framework, settings: StampConfiguration) {
        self.stamp_configuration = settings;
    }

    fn current_stamp(&self) -> &Stamp {
        self.stamps
            .get(self.current_stamp)
            .expect("Could not find the given index in stamp array")
    }
}

impl BrushEngine for StrokingEngine {
    fn stroke(
        &mut self,
        path: StrokePath,
        context: StrokeContext,
    ) -> Option<Box<dyn EditorCommand>> {
        let layer = context.editor.document().current_layer();
        match layer.layer_type {
            // TODO: Deal with difference between current_layer and buffer_layer size
            LayerType::Bitmap(ref current_layer) => {
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
                    context.renderer.begin(&current_layer.camera(), None);
                    context.renderer.draw(DrawCommand {
                        primitives: PrimitiveType::Texture2D {
                            texture_id: stamp.clone(),
                            instances: transforms,
                            flip_uv_y: true,
                            multiply_color: self.settings().wgpu_color(),
                        },
                        draw_mode: DrawMode::Instanced(0),
                        additional_data: Default::default(),
                    });
                    context.renderer.end_on_texture(current_layer.texture());
                }
            }
        }
        None
    }

    fn end_stroking(
        &mut self,
        context: &mut crate::tools::EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        None
    }

    fn begin_stroking(&mut self, context: &mut EditorContext) -> Option<Box<dyn EditorCommand>> {
        let modified_layer = context.image_editor.document().current_layer_index();
        let (old_layer_texture_id, new_texture_id) = {
            let layer = context.image_editor.document().get_layer(&modified_layer);
            let (old_layer_texture_id, layer_tex) = match layer.layer_type {
                LayerType::Bitmap(ref bm) => (bm.texture().clone(), bm),
            };
            let (width, height, format) = {
                let (width, height) = context
                    .image_editor
                    .framework()
                    .texture2d_dimensions(&old_layer_texture_id);
                let format = context
                    .image_editor
                    .framework()
                    .texture2d_format(&old_layer_texture_id);
                (width, height, format)
            };
            let new_texture_id = context.image_editor.framework().allocate_texture2d(
                framework::Texture2dConfiguration {
                    debug_name: Some("Layer".to_owned()),
                    width,
                    height,
                    format,
                    allow_cpu_write: true,
                    allow_cpu_read: true,
                    allow_use_as_render_target: true,
                },
                None,
            );

            layer_tex.draw(
                context.renderer,
                point2(0.0, 0.0),
                vec2(1.0, 1.0),
                0.0,
                1.0,
                &new_texture_id,
            );

            (old_layer_texture_id, new_texture_id)
        };
        context.image_editor.mutate_document(|doc| {
            doc.mutate_layer(&modified_layer, |lay| match &mut lay.layer_type {
                LayerType::Bitmap(bm) => bm.replace_texture(new_texture_id.clone()),
            })
        });
        let cmd = LayerReplaceCommand::new(
            context,
            context.image_editor.document().current_layer_index(),
            old_layer_texture_id,
        );
        Some(Box::new(cmd))
    }
}
