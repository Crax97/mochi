use std::fmt::Display;

use crate::tools::{EditorContext, PointerEvent};
use cgmath::{EuclideanSpace, Point2};

use framework::Box2d;
use image_editor::selection::{SelectionAddition, SelectionShape, Shape};

use super::{tool::Tool, DynamicToolUiHelpers, EditorCommand};
use strum_macros::EnumIter;

#[derive(Clone, Copy, Debug, EnumIter)]
enum SelectionShapeUi {
    Rectangle = 0,
}

impl From<usize> for SelectionShapeUi {
    fn from(v: usize) -> Self {
        match v {
            0 => Self::Rectangle,
            _ => unreachable!(),
        }
    }
}
impl From<SelectionShapeUi> for usize {
    fn from(v: SelectionShapeUi) -> Self {
        v as usize
    }
}
impl Display for SelectionShapeUi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SelectionShapeUi::Rectangle => "Rectangle",
        })
    }
}

pub struct RectSelectionTool {
    is_active: bool,
    first_click_position: Point2<f32>,
    last_click_position: Point2<f32>,
    selection_shape_ui: SelectionShapeUi,
    selection_addition: SelectionAddition,
}

impl RectSelectionTool {
    pub fn new() -> Self {
        Self {
            is_active: false,
            first_click_position: Point2::origin(),
            last_click_position: Point2::origin(),
            selection_shape_ui: SelectionShapeUi::Rectangle,
            selection_addition: SelectionAddition::Add,
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
        if !self.is_active {
            return None;
        }
        let new_position = pointer_event.new_pointer_location_normalized;
        let new_position = context
            .image_editor
            .transform_point_into_pixel_position(new_position);
        match new_position {
            Some(new_pos) => {
                self.last_click_position = new_pos;
            }
            _ => {}
        };
        let rect = Box2d::from_points(self.first_click_position, self.last_click_position);
        context.image_editor.mutate_document(|doc| {
            doc.mutate_partial_selection(|selection| {
                selection.set(SelectionShape {
                    shape: Shape::Rectangle(rect),
                    mode: self.selection_addition.clone(),
                })
            });
        });
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
            doc.mutate_selection(|selection| {
                selection.extend(SelectionShape {
                    shape: Shape::Rectangle(rect),
                    mode: self.selection_addition.clone(),
                })
            });
        });
        None
    }
    fn ui(&mut self, ui: &mut dyn super::DynamicToolUi) {
        self.selection_shape_ui =
            DynamicToolUiHelpers::dropdown(ui, "Selection shape", self.selection_shape_ui);
        self.selection_addition =
            DynamicToolUiHelpers::dropdown(ui, "Selection mode", self.selection_addition);
    }
    fn name(&self) -> &'static str {
        "Rect Selection tool"
    }
}
