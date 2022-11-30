use std::f32::consts::PI;

use crate::tools::EditorContext;
use cgmath::{point2, InnerSpace, Point2};
use strum_macros::{Display, EnumIter, EnumString};

use super::{dynamic_tool_ui_helpers, tool::Tool, DynamicToolUi, EditorCommand, PointerEvent};

#[derive(Clone, Copy, Debug, EnumIter, EnumString, Display, PartialEq, Eq)]
enum TransformItem {
    Layer = 0,
    Selection = 1,
}

impl From<usize> for TransformItem {
    fn from(v: usize) -> Self {
        match v {
            0 => Self::Layer,
            1 => Self::Selection,
            _ => unreachable!(),
        }
    }
}
impl From<TransformItem> for usize {
    fn from(v: TransformItem) -> Self {
        v as usize
    }
}
pub struct TransformLayerTool {
    is_active: bool,
    last_frame_position: Point2<f32>,
    transform_item: TransformItem,
    extract_selection: bool,
    is_manipulating_selection: bool,
}

impl TransformLayerTool {
    pub fn new() -> Self {
        Self {
            is_active: false,
            last_frame_position: point2(0.0, 0.0),
            transform_item: TransformItem::Layer,
            extract_selection: false,
            is_manipulating_selection: false,
        }
    }
}

impl Tool for TransformLayerTool {
    fn on_pointer_click(
        &mut self,
        event: PointerEvent,
        _: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        self.is_active = true;
        self.last_frame_position = event.new_pointer_location;
        None
    }

    fn on_pointer_move(
        &mut self,
        pointer_motion: super::tool::PointerEvent,
        context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        if !self.is_active {
            return None;
        }

        if self.extract_selection {
            self.extract_selection = false;
            context.image_editor.mutate_document(|doc| {
                if doc.selection_layer().is_some() {
                    doc.apply_selection(context.renderer, context.framework);
                } else {
                    doc.extract_selection(context.renderer, context.framework);
                    self.is_manipulating_selection = true;
                }
            });
        }

        let new_position = pointer_motion.new_pointer_location;
        let delta = new_position - self.last_frame_position;
        if delta.magnitude2() > 0.5 {
            context
                .image_editor
                .mutate_document(|doc| match self.transform_item {
                    TransformItem::Layer => {
                        if let Some(layer) = doc.current_layer_index().copied() {
                            doc.mutate_layer(&layer, |layer| {
                                layer.translate(delta);
                            });
                        }
                    }
                    TransformItem::Selection => {
                        if let Some(selection) = doc.selection_layer_mut() {
                            selection.layer.translate(delta);
                        } else {
                            doc.mutate_selection(|sel| sel.translate(delta))
                        }
                    }
                });
            self.last_frame_position = new_position;
        }
        None
    }

    fn on_pointer_release(
        &mut self,
        _pointer_release: PointerEvent,
        _context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        self.is_active = false;
        None
    }

    fn on_deselected(&mut self, context: &mut EditorContext) -> Option<Box<dyn EditorCommand>> {
        if self.is_manipulating_selection {
            self.is_manipulating_selection = false;
            context.image_editor.mutate_document(|doc| {
                doc.apply_selection(context.renderer, context.framework);
            });
        }
        None
    }

    fn ui(&mut self, ui: &mut dyn DynamicToolUi, context: &mut EditorContext) {
        self.transform_item =
            dynamic_tool_ui_helpers::dropdown(ui, "Transform item", self.transform_item);
        if self.is_manipulating_selection {
            if ui.button("Apply selection") {
                self.extract_selection = true;
                self.is_manipulating_selection = false;
            }
        } else {
            if ui.button("Manipulate selection") {
                self.extract_selection = true;
            }
        }

        context.image_editor.mutate_current_layer(|current_layer| {
            let current_layer_transform = current_layer.transform();
            let new_rotation = ui.value_float_ranged(
                "Layer rotation",
                current_layer_transform.rotation_radians.0,
                -PI..=PI,
            );
            let mut scale = current_layer_transform.scale.clone();
            ui.vec2_ranged("Layer scale", &mut scale, 0.1..=f32::MAX, 0.1..=f32::MAX);

            current_layer.set_rotation(new_rotation);
            current_layer.set_scale(scale);
        })
    }
    fn name(&self) -> &'static str {
        "Transform Tool"
    }
}
