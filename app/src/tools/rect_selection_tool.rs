use std::borrow::Cow;

use crate::tools::{EditorContext, PointerEvent};
use cgmath::{vec2, EuclideanSpace, Point2};
use framework::{
    framework::{DepthStencilTextureId, ShaderId, TextureId},
    renderer::draw_command::{DrawCommand, DrawMode, OptionalDrawData, PrimitiveType},
    shader::{BindElement, ShaderCreationInfo},
    Box2d, Camera2d, DepthStencilTextureConfiguration, Framework, Texture2dConfiguration,
    Transform2d,
};
use image::{DynamicImage, RgbaImage};
use image_editor::{
    layers::{BitmapLayer, BitmapLayerConfiguration, Layer, LayerCreationInfo},
    LayerConstructionInfo,
};
use wgpu::{
    DepthBiasState, DepthStencilState, ShaderModuleDescriptor, StencilFaceState, StencilState,
};

use super::{tool::Tool, EditorCommand};

pub struct RectSelectionTool {
    is_active: bool,
    first_click_position: Point2<f32>,
    last_click_position: Point2<f32>,

    draw_on_stencil_buffer_shader_id: ShaderId,
    draw_masked_stencil_buffer_shader_id: ShaderId,
    draw_masked_inverted_stencil_buffer_shader_id: ShaderId,
    dotted_shader: ShaderId,
}

impl RectSelectionTool {
    pub fn new(framework: &Framework) -> Self {
        let info = ShaderCreationInfo::using_default_vertex_fragment(framework).with_depth_state(
            Some(DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: StencilState {
                    front: StencilFaceState {
                        compare: wgpu::CompareFunction::Always,
                        pass_op: wgpu::StencilOperation::Replace,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                    },
                    back: StencilFaceState {
                        compare: wgpu::CompareFunction::Always,
                        pass_op: wgpu::StencilOperation::Replace,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                    },
                    read_mask: 0xFFFFFFF,
                    write_mask: 0xFFFFFFF,
                },
                bias: DepthBiasState::default(),
            }),
        );
        let draw_on_stencil_buffer_shader_id = framework.create_shader(info);
        let info = ShaderCreationInfo::using_default_vertex_fragment(framework).with_depth_state(
            Some(DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: StencilState {
                    front: StencilFaceState {
                        compare: wgpu::CompareFunction::Equal,
                        pass_op: wgpu::StencilOperation::Keep,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                    },
                    back: StencilFaceState {
                        compare: wgpu::CompareFunction::Equal,
                        pass_op: wgpu::StencilOperation::Keep,
                        fail_op: wgpu::StencilOperation::Keep,
                        depth_fail_op: wgpu::StencilOperation::Keep,
                    },
                    read_mask: 0xFFFFFFF,
                    write_mask: 0xFFFFFFF,
                },
                bias: DepthBiasState::default(),
            }),
        );
        let draw_masked_stencil_buffer_shader_id = framework.create_shader(info);

        let info = ShaderCreationInfo::using_default_vertex_fragment(framework).with_depth_state(
            Some(DepthStencilState {
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
            }),
        );
        let draw_masked_inverted_stencil_buffer_shader_id = framework.create_shader(info);

        let dotted_module_descriptor = framework
            .shader_compiler
            .compile_into_shader_description("Dotted shader", include_str!("dotted_selection.wgsl"))
            .unwrap();
        let dotted_info =
            ShaderCreationInfo::using_default_vertex(framework, dotted_module_descriptor)
                .with_bind_element(BindElement::Texture); // 2: diffuse texture + sampler
        let dotted_shader = framework.create_shader(dotted_info);
        Self {
            is_active: false,
            first_click_position: Point2::origin(),
            last_click_position: Point2::origin(),
            draw_on_stencil_buffer_shader_id,
            draw_masked_stencil_buffer_shader_id,
            draw_masked_inverted_stencil_buffer_shader_id,
            dotted_shader,
        }
    }
}

impl Tool for RectSelectionTool {
    fn on_pointer_click(
        &mut self,
        event: PointerEvent,
        context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        self.is_active = true;

        self.first_click_position = context
            .image_editor
            .transform_point_into_pixel_position(event.new_pointer_location_normalized)
            .unwrap();
        self.last_click_position = self.first_click_position.clone();
        None
    }

    fn on_pointer_move(
        &mut self,
        pointer_event: PointerEvent,
        context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        let new_position = pointer_event.new_pointer_location_normalized;
        let new_position = context
            .image_editor
            .transform_point_into_pixel_position(new_position);
        match new_position {
            Some(new_pos) => {
                self.last_click_position = new_pos;
            }
            _ => {}
        }
        None
    }

    fn on_pointer_release(
        &mut self,
        _pointer_event: PointerEvent,
        context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        self.is_active = false;

        let current_layer = context.image_editor.document().current_layer();
        let framework = context.image_editor.framework();
        let (width, height) = context
            .image_editor
            .framework()
            .texture2d_dimensions(current_layer.bitmap.texture());
        let stencil_texture =
            framework.allocate_depth_stencil_texture(DepthStencilTextureConfiguration {
                debug_name: Some("Selection tool depth stencil texture"),
                width,
                height,
                is_stencil: true,
            });
        let format = context
            .image_editor
            .framework()
            .texture2d_format(current_layer.bitmap.texture());
        let new_texture = context.image_editor.framework().allocate_texture2d(
            Texture2dConfiguration {
                debug_name: None,
                width,
                height,
                format,
                allow_cpu_write: true,
                allow_cpu_read: true,
                allow_use_as_render_target: true,
            },
            None,
        );
        let old_texture_copy = context.image_editor.framework().allocate_texture2d(
            Texture2dConfiguration {
                debug_name: None,
                width,
                height,
                format,
                allow_cpu_write: true,
                allow_cpu_read: true,
                allow_use_as_render_target: true,
            },
            None,
        );

        // 1. Draw the selection rect on the stencil buffer
        context.renderer.begin(
            &current_layer.bitmap.camera(),
            Some(wgpu::Color::TRANSPARENT),
        );
        context.renderer.set_stencil_clear(Some(0));
        context.renderer.set_stencil_reference(255);
        let rect = Box2d::from_points(self.first_click_position, self.last_click_position);
        context.renderer.draw(DrawCommand {
            primitives: PrimitiveType::Rect {
                rects: vec![rect],
                multiply_color: wgpu::Color::GREEN,
            },
            draw_mode: DrawMode::Single,
            additional_data: OptionalDrawData::just_shader(Some(
                self.draw_on_stencil_buffer_shader_id.clone(),
            )),
        });
        context
            .renderer
            .end_on_texture(&new_texture, Some(&stencil_texture));
        // 2. Draw layer using the rect stencil buffer, this is the selection. Store it into a new texture
        context.renderer.begin(
            &current_layer.bitmap.camera(),
            Some(wgpu::Color::TRANSPARENT),
        );
        context.renderer.set_stencil_clear(None);
        context.renderer.set_stencil_reference(255);
        context.renderer.draw(DrawCommand {
            primitives: PrimitiveType::Texture2D {
                texture_id: current_layer.bitmap.texture().clone(),
                instances: vec![current_layer.pixel_transform()],
                flip_uv_y: true,
                multiply_color: wgpu::Color::WHITE,
            },
            draw_mode: DrawMode::Single,
            additional_data: OptionalDrawData::just_shader(Some(
                self.draw_masked_stencil_buffer_shader_id.clone(),
            )),
        });
        context
            .renderer
            .end_on_texture(&new_texture, Some(&stencil_texture));
        // 3. Draw the layer using the inverted stencil buffer: this is the remaining part of the texture

        context.renderer.begin(
            &current_layer.bitmap.camera(),
            Some(wgpu::Color::TRANSPARENT),
        );
        context.renderer.set_stencil_clear(None);
        context.renderer.set_stencil_reference(255);
        context.renderer.draw(DrawCommand {
            primitives: PrimitiveType::Texture2D {
                texture_id: current_layer.bitmap.texture().clone(),
                instances: vec![current_layer.pixel_transform()],
                flip_uv_y: true,
                multiply_color: wgpu::Color::WHITE,
            },
            draw_mode: DrawMode::Single,
            additional_data: OptionalDrawData::just_shader(Some(
                self.draw_masked_inverted_stencil_buffer_shader_id.clone(),
            )),
        });
        context
            .renderer
            .end_on_texture(&old_texture_copy, Some(&stencil_texture));

        //5. Now add the new layer
        let (width, height) = framework.texture2d_dimensions(&new_texture);
        let new_index = context
            .image_editor
            .add_layer_to_document(LayerConstructionInfo {
                initial_color: [0.0, 0.0, 0.0, 0.0],
                name: current_layer.settings().name.clone() + " subregion",
                width,
                height,
            });
        context.image_editor.mutate_document(|doc| {
            doc.mutate_layer(&new_index, |layer| {
                layer.replace_texture(new_texture.clone())
            });
            doc.mutate_layer(&doc.current_layer_index(), |layer| {
                layer.replace_texture(old_texture_copy.clone())
            });
            doc.select_layer(new_index);
        });
        None
    }

    fn draw(&self, renderer: &mut framework::renderer::renderer::Renderer) {
        if !self.is_active {
            return;
        }
        let rect = Box2d::from_points(self.first_click_position, self.last_click_position);
        renderer.draw(DrawCommand {
            primitives: PrimitiveType::Rect {
                rects: vec![rect],
                multiply_color: wgpu::Color::RED,
            },
            draw_mode: DrawMode::Single,
            additional_data: OptionalDrawData::just_shader(Some(self.dotted_shader.clone())),
        });
    }
    fn name(&self) -> &'static str {
        "Rect Selection tool"
    }
}
