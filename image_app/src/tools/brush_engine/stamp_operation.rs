use cgmath::{point2, point3, vec2, EuclideanSpace, Matrix4, Point2, Rad, SquareMatrix, Transform};
use framework::{
    framework::{BufferId, ShaderId, TextureId},
    renderer::{
        draw_command::{BindableResource, DrawCommand, DrawMode, OptionalDrawData, PrimitiveType},
        renderer::Renderer,
    },
    Camera2d, Framework, Transform2d,
};
use image_editor::layers::{ChunkDiff, LayerOperation, OperationResult};

use super::StrokePath;

pub(crate) struct StampOperation {
    pub path: StrokePath,
    pub brush: TextureId,
    pub color: wgpu::Color,
    pub is_eraser: bool,
    pub brush_settings_buffer: BufferId,

    pub eraser_shader_id: ShaderId,
    pub brush_shader_id: ShaderId,

    pub diff: ChunkDiff,
}

impl LayerOperation for StampOperation {
    fn execute(
        &mut self,
        layer: &mut image_editor::layers::Layer,
        bounds: framework::Box2d,
        renderer: &mut Renderer,
        framework: &mut Framework,
    ) -> image_editor::layers::OperationResult {
        let layer_transform = layer.transform();
        let inv_layer_matrix = layer_transform.matrix().invert();
        let layer_rendering_camera = layer.rendering_camera();

        if let (Some(inv_layer_matrix), Some(rendering_camera)) =
            (inv_layer_matrix, layer_rendering_camera)
        {
            match &mut layer.layer_type {
                image_editor::layers::LayerType::Chonky(map) => {
                    let chunk_size = map.chunk_size();
                    let bounds = bounds.transformed(inv_layer_matrix);
                    self.diff = map.edit(
                        bounds,
                        |chunk, _, chunk_world_position, framework| {
                            self.stamp_on_texture(
                                inv_layer_matrix,
                                chunk_world_position,
                                renderer,
                                rendering_camera,
                                framework,
                                chunk_size,
                                chunk_size,
                                chunk,
                            );
                        },
                        framework,
                    );
                }
                _ => unreachable!(),
            }
        }
        OperationResult::Rerender
    }

    fn accept(&self, layer: &image_editor::layers::Layer) -> bool {
        match &layer.layer_type {
            image_editor::layers::LayerType::Chonky(_) => true,
            _ => false,
        }
    }
}

impl StampOperation {
    fn stamp_on_texture(
        &self,
        inv_layer_matrix: Matrix4<f32>,
        offset: Point2<f32>,
        renderer: &mut Renderer,
        camera_to_use: Camera2d,
        framework: &mut Framework,
        width: u32,
        height: u32,
        texture: &TextureId,
    ) {
        let transforms: Vec<Transform2d> = self
            .path
            .points
            .iter()
            .map(|pt| {
                /*
                    Explanation:
                    This works because the chunks we stroke to are the ones that gets selected from the transformed
                    bounds, so we don't need to transform the offset as well: it's already transformed
                */
                let stroke_origin =
                    inv_layer_matrix.transform_point(point3(pt.position.x, pt.position.y, 0.0));
                let stroke_origin = stroke_origin - point3(offset.x, offset.y, 0.0).to_vec();
                Transform2d {
                    position: stroke_origin,
                    scale: vec2(pt.size, pt.size),
                    rotation_radians: Rad(0.0),
                }
            })
            .collect();
        // 2. Do draw
        let stamp = self.brush.clone();
        renderer.begin(&camera_to_use, None, framework);
        renderer.set_viewport(Some((0.0, 0.0, width as f32, height as f32)));
        renderer.draw(DrawCommand {
            primitives: PrimitiveType::Texture2D {
                texture_id: stamp,
                instances: transforms,
                flip_uv_y: true,
                multiply_color: self.color,
            },
            draw_mode: DrawMode::Instanced,
            additional_data: OptionalDrawData {
                shader: Some(if self.is_eraser {
                    self.eraser_shader_id.clone()
                } else {
                    self.brush_shader_id.clone()
                }),
                additional_bindable_resource: vec![BindableResource::UniformBuffer(
                    self.brush_settings_buffer.clone(),
                )],
                ..Default::default()
            },
        });
        renderer.end(texture, None, framework);
    }

    pub(crate) fn diff(self) -> ChunkDiff {
        self.diff
    }
}
