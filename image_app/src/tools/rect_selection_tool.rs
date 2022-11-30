use std::fmt::Display;

use crate::tools::{EditorContext, PointerEvent};
use cgmath::{EuclideanSpace, Point2};

use framework::Box2d;
use image_editor::selection::{SelectionAddition, SelectionShape, Shape};

use super::{dynamic_tool_ui_helpers, tool::Tool, EditorCommand};
use strum_macros::{Display, EnumIter, EnumString};

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

#[derive(Clone, Copy, Debug, EnumIter, EnumString, Display, PartialEq, Eq)]
enum SelectionEditMode {
    Edit = 0,
    Traslate = 1,
}

impl From<usize> for SelectionEditMode {
    fn from(v: usize) -> Self {
        match v {
            0 => Self::Edit,
            1 => Self::Traslate,
            _ => unreachable!(),
        }
    }
}
impl From<SelectionEditMode> for usize {
    fn from(v: SelectionEditMode) -> Self {
        v as usize
    }
}
pub struct RectSelectionTool {
    is_active: bool,
    first_click_position: Point2<f32>,
    last_click_position: Point2<f32>,
    selection_shape_ui: SelectionShapeUi,
    selection_addition: SelectionAddition,
    edit_mode: SelectionEditMode,
}

impl RectSelectionTool {
    pub fn new() -> Self {
        Self {
            is_active: false,
            first_click_position: Point2::origin(),
            last_click_position: Point2::origin(),
            selection_shape_ui: SelectionShapeUi::Rectangle,
            selection_addition: SelectionAddition::Add,
            edit_mode: SelectionEditMode::Edit,
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
        let delta = new_position.unwrap() - self.last_click_position;
        match new_position {
            Some(new_pos) => {
                self.last_click_position = new_pos;
            }
            _ => {}
        };
        let rect = Box2d::from_points(self.first_click_position, self.last_click_position);
        context.image_editor.mutate_document(|doc| {
            match self.edit_mode {
                SelectionEditMode::Edit => doc.mutate_partial_selection(|selection| {
                    selection.set(SelectionShape {
                        shape: Shape::Rectangle(rect),
                        mode: self.selection_addition.clone(),
                    })
                }),
                SelectionEditMode::Traslate => {
                    doc.mutate_selection(|selection| selection.translate(delta))
                }
            };
        });
        None
    }

    fn on_pointer_release(
        &mut self,
        _pointer_event: PointerEvent,
        context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        self.is_active = false;

        if self.edit_mode == SelectionEditMode::Traslate {
            return None; // Do nothing when traslating the selection
        }

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

    fn ui(&mut self, ui: &mut dyn super::DynamicToolUi, _: &mut EditorContext) {
        self.selection_shape_ui =
            dynamic_tool_ui_helpers::dropdown(ui, "Selection shape", self.selection_shape_ui);
        self.selection_addition =
            dynamic_tool_ui_helpers::dropdown(ui, "Selection mode", self.selection_addition);
        self.edit_mode = dynamic_tool_ui_helpers::dropdown(ui, "Edit mode", self.edit_mode);
    }
    fn name(&self) -> &'static str {
        "Rect Selection tool"
    }
}
