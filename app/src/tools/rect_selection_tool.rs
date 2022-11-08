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
}

impl RectSelectionTool {
    pub fn new(framework: &Framework) -> Self {
        Self {
            is_active: false,
            first_click_position: Point2::origin(),
            last_click_position: Point2::origin(),
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

        let rect = Box2d::from_points(self.first_click_position, self.last_click_position);

        context.image_editor.mutate_document(|doc| {
            doc.copy_layer_selection_to_new_layer(context.renderer, rect);
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
            additional_data: OptionalDrawData::just_shader(Some(
                image_editor::global_selection_data().dotted_shader.clone(),
            )),
        });
    }
    fn name(&self) -> &'static str {
        "Rect Selection tool"
    }
}
