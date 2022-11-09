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
    selection::SelectionShape,
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
        context
            .image_editor
            .document()
            .draw_selection(context.renderer);
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
            doc.mutate_selection(|selection| selection.set(SelectionShape::Rectangle(rect)));
        });
        None
    }

    fn draw(&self, renderer: &mut framework::renderer::renderer::Renderer) {}
    fn name(&self) -> &'static str {
        "Rect Selection tool"
    }
}
