use std::f32::consts::FRAC_PI_2;

use cgmath::{point3, vec2, EuclideanSpace, Matrix2, Point2, Rad, SquareMatrix, Transform};
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
                                layer_transform,
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
        layer_transform: Transform2d,
        chunk_world_offset: Point2<f32>,
        renderer: &mut Renderer,
        camera_to_use: Camera2d,
        framework: &mut Framework,
        target_width: u32,
        target_height: u32,
        stamp_texture: &TextureId,
    ) {
        let transforms = self.make_stamp_transforms(layer_transform, chunk_world_offset);
        self.stamp_on_chunk(
            renderer,
            camera_to_use,
            framework,
            target_width,
            target_height,
            transforms,
            stamp_texture,
        );
    }

    fn stamp_on_chunk(
        &self,
        renderer: &mut Renderer,
        camera_to_use: Camera2d,
        framework: &mut Framework,
        target_width: u32,
        target_height: u32,
        transforms: Vec<Transform2d>,
        stamp_texture: &framework::AssetId<
            framework::GpuTexture<framework::RgbaU8, framework::Texture2D<framework::RgbaU8>>,
        >,
    ) {
        let stamp = self.brush.clone();
        renderer.begin(&camera_to_use, None, framework);
        renderer.set_viewport(Some((0.0, 0.0, target_width as f32, target_height as f32)));
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
        renderer.end(stamp_texture, None, framework);
    }

    fn make_stamp_transforms(
        &self,
        layer_transform: Transform2d,
        chunk_world_offset: Point2<f32>,
    ) -> Vec<Transform2d> {
        let inv_layer_matrix = layer_transform.matrix().invert().unwrap();
        let transforms: Vec<Transform2d> = self
            .path
            .points
            .iter()
            .map(|pt| {
                let stroke_scale = vec2(pt.size, pt.size);
                let stroke_origin =
                    inv_layer_matrix.transform_point(point3(pt.position.x, pt.position.y, 0.0));

                /*
                    Explanation:
                    This works because the chunks we stroke to are the ones that gets selected from the transformed
                    bounds, so we don't need to transform the offset as well: it's already transformed
                */
                let stroke_origin = stroke_origin
                    - point3(chunk_world_offset.x, chunk_world_offset.y, 0.0).to_vec();
                Transform2d {
                    position: stroke_origin,
                    scale: stroke_scale, // Account for layer scale when stamping
                    rotation_radians: Rad(0.0),
                }
            })
            .collect();
        transforms
    }

    pub(crate) fn diff(self) -> ChunkDiff {
        self.diff
    }
}
