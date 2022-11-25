use cgmath::{point3, vec2, Rad, SquareMatrix, Transform};
use framework::{
    framework::{BufferId, ShaderId, TextureId},
    renderer::{
        draw_command::{BindableResource, DrawCommand, DrawMode, OptionalDrawData, PrimitiveType},
        renderer::Renderer,
    },
    Framework, Transform2d,
};
use image_editor::{document::Document, layers::ImageLayerOperation};

use super::{stamping_engine::StrokingEngine, StrokePath};

pub(crate) struct StampOperation {
    pub path: StrokePath,
    pub brush: TextureId,
    pub color: wgpu::Color,
    pub is_eraser: bool,
    pub brush_settings_buffer: BufferId,

    pub eraser_shader_id: ShaderId,
    pub brush_shader_id: ShaderId,
}

impl ImageLayerOperation for StampOperation {
    fn image_op(
        &self,
        image_texture: &TextureId,
        dimensions: &cgmath::Vector2<u32>,
        owning_layer: &image_editor::layers::Layer,
        renderer: &mut Renderer,
        framework: &mut Framework,
    ) -> image_editor::layers::OperationResult {
        // 1. Create draw info

        let current_layer_transform = owning_layer.transform();
        let inv_layer_matrix = current_layer_transform.matrix().invert();
        if let Some(inv_layer_matrix) = inv_layer_matrix {
            let transforms: Vec<Transform2d> = self
                .path
                .points
                .iter()
                .map(|pt| {
                    let origin_inv =
                        inv_layer_matrix.transform_point(point3(pt.position.x, pt.position.y, 0.0));
                    Transform2d {
                        position: origin_inv,
                        scale: vec2(pt.size, pt.size),
                        rotation_radians: Rad(0.0),
                    }
                })
                .collect();

            // 2. Do draw

            let stamp = self.brush.clone();
            renderer.begin(
                &Document::make_camera_for_layer(owning_layer),
                None,
                framework,
            );
            renderer.set_viewport(Some((0.0, 0.0, dimensions.x as f32, dimensions.y as f32)));
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
            renderer.end(image_texture, None, framework);
        }
        image_editor::layers::OperationResult::Rerender
    }
}
