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
        new_texture_id: TextureId,
    ) -> Self {
        let old_layer_texture_id = match context
            .image_editor
            .document()
            .get_layer(&modified_layer)
            .layer_type
        {
            LayerType::Bitmap(ref bm) => bm.texture().clone(),
        };
        context.image_editor.mutate_document(|doc| {
            doc.mutate_layer(&modified_layer, |lay| match &mut lay.layer_type {
                LayerType::Bitmap(bm) => bm.replace_texture(new_texture_id.clone()),
            })
        });

        Self {
            old_layer_texture_id,
            modified_layer,
        }
    }
}

impl EditorCommand for LayerReplaceCommand {
    fn undo(&self, context: &mut EditorContext) -> Box<dyn EditorCommand> {
        Box::new(LayerReplaceCommand::new(
            context,
            self.modified_layer,
            self.old_layer_texture_id.clone(),
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
    camera_buffer_id: BufferId,
    stamp_configuration: StampConfiguration,
}

impl StrokingEngine {
    pub fn new(initial_stamp: Stamp, framework: &Framework) -> Self {
        let camera_buffer_id = framework.allocate_typed_buffer(BufferConfiguration::<
            Camera2dUniformBlock,
        > {
            initial_setup: framework::buffer::BufferInitialSetup::Size(std::mem::size_of::<
                Camera2dUniformBlock,
            >() as u64),
            buffer_type: framework::BufferType::Uniform,
            allow_write: true,
            allow_read: false,
        });

        Self {
            stamps: vec![initial_stamp],
            current_stamp: 0,
            camera_buffer_id,
            stamp_configuration: StampConfiguration {
                color_srgb: [255, 255, 255],
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
    fn stroke(&mut self, path: StrokePath, context: StrokeContext) {
        match context.editor.document().current_layer().layer_type {
            // TODO: Deal with difference between current_layer and buffer_layer size
            LayerType::Bitmap(_) => {
                let buffer_layer = context.editor.document().buffer_layer();

                // 1. Update buffer
                let transforms: Vec<Transform2d> = path
                    .points
                    .iter()
                    .map(|pt| Transform2d {
                        position: point3(pt.position.x, pt.position.y, 0.0),
                        scale: vec2(pt.size, pt.size),
                        rotation_radians: Rad(0.0),
                    })
                    .collect();

                // 2. Do draw

                let stamp = self.current_stamp().brush_texture.texture();
                context.renderer.begin(&buffer_layer.camera(), None);
                context.renderer.draw(DrawCommand {
                    primitives: PrimitiveType::Texture2D {
                        texture_id: stamp.clone(),
                        instances: transforms,
                        flip_uv_y: true,
                    },
                    draw_mode: DrawMode::Instanced(0),
                    additional_data: Default::default(),
                });
                context.renderer.end_on_texture(buffer_layer.texture());
            }
        }
    }

    fn end_stroking(
        &mut self,
        context: &mut crate::tools::EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        let new_texture_id = {
            let modified_layer = context.image_editor.document().current_layer_index();
            let layer = context.image_editor.document().get_layer(&modified_layer);
            let (old_layer_texture_id, size, layer_tex) = match layer.layer_type {
                LayerType::Bitmap(ref bm) => (bm.texture(), bm.size(), bm),
            };
            let (width, height, format) = {
                let (width, height) = context
                    .image_editor
                    .framework()
                    .texture2d_dimensions(old_layer_texture_id);
                let format = context
                    .image_editor
                    .framework()
                    .texture2d_format(old_layer_texture_id);
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

            let current_layer_transform = layer.transform();
            // The buffer_layer is always drawn in front of the camera, so to correctly blend it with
            // The current layer may be placed away from the camera
            // To correctly blend the buffer with the current layer, we have to move into the current layer's
            // coordinate system
            let inv_layer_matrix = current_layer_transform.matrix().invert();
            let buffer_layer = context.image_editor.document().buffer_layer();
            if let Some(inv_layer_matrix) = inv_layer_matrix {
                let origin_inv = inv_layer_matrix.transform_point(point3(0.0, 0.0, 0.0));
                context.renderer.begin(&layer_tex.camera(), None);
                layer_tex.draw(
                    context.renderer,
                    point2(0.0, 0.0),
                    vec2(1.0, 1.0),
                    0.0,
                    1.0,
                    &new_texture_id,
                );
                buffer_layer.draw(
                    context.renderer,
                    point2(origin_inv.x, origin_inv.y),
                    vec2(1.0, 1.0),
                    0.0,
                    1.0,
                    &new_texture_id,
                );

                context
                    .renderer
                    .begin(&layer_tex.camera(), Some(wgpu::Color::TRANSPARENT));
                context.renderer.end_on_texture(buffer_layer.texture());
            }
            new_texture_id
        };
        let cmd = LayerReplaceCommand::new(
            context,
            context.image_editor.document().current_layer_index(),
            new_texture_id,
        );

        context
            .image_editor
            .document()
            .blend_buffer_onto_current_layer();
        Some(Box::new(cmd))
    }
}
