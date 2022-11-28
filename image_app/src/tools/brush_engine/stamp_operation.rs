use cgmath::{point3, vec2, Rad, SquareMatrix, Transform};
use framework::{
    framework::{BufferId, ShaderId, TextureId},
    renderer::{
        draw_command::{BindableResource, DrawCommand, DrawMode, OptionalDrawData, PrimitiveType},
        renderer::Renderer,
    },
    Camera2d, Framework, Transform2d,
};
use image_editor::{
    document::Document,
    layers::{ImageLayerOperation, LayerOperation, OperationResult},
};

use super::StrokePath;

pub(crate) struct StampOperation {
    pub path: StrokePath,
    pub brush: TextureId,
    pub color: wgpu::Color,
    pub is_eraser: bool,
    pub brush_settings_buffer: BufferId,

    pub eraser_shader_id: ShaderId,
    pub brush_shader_id: ShaderId,
}

impl LayerOperation for StampOperation {
    fn execute(
        &self,
        layer: &mut image_editor::layers::Layer,
        bounds: framework::Box2d,
        renderer: &mut Renderer,
        framework: &mut Framework,
    ) -> image_editor::layers::OperationResult {
        let layer_transform = layer.transform();

        match &mut layer.layer_type {
            image_editor::layers::LayerType::Chonky(map) => {
                let chunk_size = map.chunk_size();
                let chunk_camera = Camera2d::wh(chunk_size, chunk_size);
                map.edit(
                    bounds,
                    |chunk, _, chunk_world_position, framework| {
                        let inv_layer_matrix = layer_transform.matrix().invert();
                        if let Some(inv_layer_matrix) = inv_layer_matrix {
                            let transforms: Vec<Transform2d> = self
                                .path
                                .points
                                .iter()
                                .map(|pt| {
                                    let origin_inv = inv_layer_matrix.transform_point(point3(
                                        pt.position.x - chunk_world_position.x,
                                        pt.position.y - chunk_world_position.y,
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
                            let stamp = self.brush.clone();
                            renderer.begin(&chunk_camera, None, framework);
                            renderer.set_viewport(Some((
                                0.0,
                                0.0,
                                chunk_size as f32,
                                chunk_size as f32,
                            )));
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
                                    additional_bindable_resource: vec![
                                        BindableResource::UniformBuffer(
                                            self.brush_settings_buffer.clone(),
                                        ),
                                    ],
                                    ..Default::default()
                                },
                            });
                            renderer.end(chunk, None, framework);
                        }
                    },
                    framework,
                );
                OperationResult::Rerender
            }
            _ => unreachable!(),
        }
    }

    fn accept(&self, layer: &image_editor::layers::Layer) -> bool {
        match &layer.layer_type {
            image_editor::layers::LayerType::Chonky(_) => true,
            _ => false,
        }
    }
}
