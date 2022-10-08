use cgmath::{point2, point3, vec2, SquareMatrix, Transform, Vector2};
use framework::framework::TextureId;
use framework::{Box2d, TypedBufferConfiguration};
use framework::{Framework, MeshInstance2D, TypedBuffer};
use image_editor::layers::{BitmapLayer, LayerIndex, LayerType};
use image_editor::ImageEditor;
use scene::{Camera2d, Camera2dUniformBlock};
use wgpu::{BindGroup, RenderPassColorAttachment, RenderPassDescriptor};

use crate::tools::{EditorCommand, EditorContext};
use crate::{StrokeContext, StrokePath};

use super::stamping_engine_pass::StampingEngineRenderPass;
use super::BrushEngine;

struct BrushApplyCommand {
    modified_layer: LayerIndex,
    modified_region: Box2d,
    modified_region_old_texture_id: TextureId,
}

impl BrushApplyCommand {
    pub fn new(editor: &ImageEditor, modified_layer: LayerIndex, modified_region: Box2d) -> Self {
        let edited_layer = editor.document().get_layer(&modified_layer);
        let modified_region_old_texture_id = match edited_layer.layer_type {
            image_editor::layers::LayerType::Bitmap(ref bm) => {
                let edited_texture = bm.texture();
                let edited_texture = editor.framework().texture2d(edited_texture);
                edited_texture.read_subregion_texture2d(
                    modified_region.origin.x as u32,
                    modified_region.origin.y as u32,
                    modified_region.extents.x as u32,
                    modified_region.extents.y as u32,
                    editor.framework(),
                )
            }
        };

        Self {
            modified_layer,
            modified_region,
            modified_region_old_texture_id,
        }
    }
}

impl EditorCommand for BrushApplyCommand {
    fn execute(&self, editor_context: &mut EditorContext) {
        let framework = editor_context.image_editor.framework();
        let layer = editor_context
            .image_editor
            .document()
            .get_layer(&self.modified_layer);
        match layer.layer_type {
            LayerType::Bitmap(ref bm) => {
                let bm_camera = Camera2d::new(
                    -0.1,
                    1000.0,
                    [
                        -bm.size().x as f32 * 0.5,
                        bm.size().x as f32 * 0.5,
                        bm.size().y as f32 * 0.5,
                        -bm.size().y as f32 * 0.5,
                    ],
                );
                let output_id = bm.texture();
                let texture = framework.texture2d(output_id);

                let current_layer_transform = layer.transform();

                // The buffer_layer is always drawn in front of the camera, so to correctly blend it with
                // The current layer may be placed away from the camera
                // To correctly blend the buffer with the current layer, we have to move into the current layer's
                // coordinate system
                let inv_layer_matrix = current_layer_transform.matrix().invert();
                if let Some(inv_layer_matrix) = inv_layer_matrix {
                    let region_center = self.modified_region.center().cast::<f32>().unwrap();
                    let origin_inv = inv_layer_matrix.transform_point(point3(
                        region_center.x,
                        region_center.y,
                        0.0,
                    ));
                    editor_context.draw_pass.begin(&bm_camera);
                    let modified_region_texture =
                        framework.texture2d(&self.modified_region_old_texture_id);

                    let real_scale = Vector2 {
                        x: modified_region_texture.width() as f32 * 0.5,
                        y: modified_region_texture.height() as f32 * 0.5,
                    };

                    editor_context.draw_pass.draw_texture(
                        &self.modified_region_old_texture_id,
                        MeshInstance2D::new(
                            point2(origin_inv.x, origin_inv.y),
                            real_scale,
                            0.0,
                            false,
                            1.0,
                        ),
                    );
                    editor_context
                        .draw_pass
                        .finish(texture.texture_view(), false);

                    editor_context.draw_pass.begin(&bm_camera);
                }
            }
        }
    }

    fn undo(&self) -> Box<dyn EditorCommand> {
        todo!()
    }
}

pub struct Stamp {
    pub(crate) brush_texture: BitmapLayer,
}

pub struct StampCreationInfo<'framework> {
    pub camera_buffer: &'framework TypedBuffer<'framework>,
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

pub struct StrokingEngine<'framework> {
    current_stamp: usize,
    stamps: Vec<Stamp>,
    stamp_pass: StampingEngineRenderPass<'framework>,
    camera_buffer: TypedBuffer<'framework>,
    camera_bind_group: BindGroup,
}

impl<'framework> StrokingEngine<'framework> {
    pub fn new(initial_stamp: Stamp, framework: &'framework Framework) -> Self {
        let stamp_pass = StampingEngineRenderPass::new(framework);

        let camera_buffer =
            framework.allocate_typed_buffer(TypedBufferConfiguration::<Camera2dUniformBlock> {
                initial_setup: framework::typed_buffer::BufferInitialSetup::Size(
                    std::mem::size_of::<Camera2dUniformBlock>() as u64,
                ),
                buffer_type: framework::BufferType::Uniform,
                allow_write: true,
                allow_read: false,
            });
        let camera_bind_group_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Texture2D Camera Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });
        let camera_bind_group = framework
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Texture2D Camera"),
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        camera_buffer.inner_buffer().as_entire_buffer_binding(),
                    ),
                }],
            });

        Self {
            stamps: vec![initial_stamp],
            current_stamp: 0,
            stamp_pass,
            camera_buffer,
            camera_bind_group,
        }
    }

    pub fn create_stamp(&self, brush_texture: BitmapLayer) -> Stamp {
        Stamp::new(brush_texture)
    }

    pub fn settings(&self) -> StampConfiguration {
        self.stamp_pass.get_stamp_settings()
    }

    pub fn set_new_settings(&mut self, settings: StampConfiguration) {
        self.stamp_pass.set_stamp_settings(settings);
    }

    fn current_stamp(&self) -> &Stamp {
        self.stamps
            .get(self.current_stamp)
            .expect("Could not find the given index in stamp array")
    }
}

impl<'framework> BrushEngine for StrokingEngine<'framework> {
    fn stroke(&mut self, path: StrokePath, context: StrokeContext) {
        match context.editor.document().current_layer().layer_type {
            // TODO: Deal with difference between current_layer and buffer_layer size
            LayerType::Bitmap(_) => {
                let buffer_layer = context.editor.document().buffer_layer();

                // 1. Update buffer
                let instances: Vec<MeshInstance2D> = path
                    .points
                    .iter()
                    .map(|pt| {
                        MeshInstance2D::new(pt.position, vec2(pt.size, pt.size), 0.0, true, 1.0)
                    })
                    .collect();

                let bm_camera = Camera2d::new(
                    -0.1,
                    1000.0,
                    [
                        -buffer_layer.size().x as f32 * 0.5,
                        buffer_layer.size().x as f32 * 0.5,
                        buffer_layer.size().y as f32 * 0.5,
                        -buffer_layer.size().y as f32 * 0.5,
                    ],
                );
                self.camera_buffer
                    .write_sync::<Camera2dUniformBlock>(&vec![(&bm_camera).into()]);
                self.stamp_pass.update(instances);
                // 2. Do draw
                let bitmap_texture = context.editor.framework().texture2d(buffer_layer.texture());
                let stroking_engine_render_pass = RenderPassDescriptor {
                    label: Some("Stamping Engine render pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: bitmap_texture.texture_view(),
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                };
                let mut render_pass = context
                    .command_encoder
                    .begin_render_pass(&stroking_engine_render_pass);

                render_pass.set_viewport(
                    0.0,
                    0.0,
                    buffer_layer.size().x,
                    buffer_layer.size().y,
                    0.0,
                    1.0,
                );
                let stamp = self.current_stamp().brush_texture.texture();
                let stamp = context.editor.framework().texture2d(stamp);
                self.stamp_pass.execute(
                    render_pass,
                    &stamp,
                    &context.editor.framework().asset_library,
                    &self.camera_bind_group,
                );
            }
        }
    }

    fn end_stroking(
        &mut self,
        context: &mut crate::tools::EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        context
            .image_editor
            .document()
            .blend_buffer_onto_current_layer(context.draw_pass);
        None
    }
}
