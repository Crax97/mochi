use std::ops::RangeInclusive;

use cgmath::{Point2, Vector2};
use framework::{renderer::renderer::Renderer, Framework};

use crate::EditorCommand;
use image_editor::ImageEditor;

pub struct EditorContext<'editor> {
    pub framework: &'editor mut Framework,
    pub image_editor: &'editor mut ImageEditor,
    pub renderer: &'editor mut Renderer,
}

#[derive(Debug, Clone, Copy)]
pub struct PointerEvent {
    pub new_pointer_location_normalized: Point2<f32>,
    pub new_pointer_location: Point2<f32>,
    pub pressure: f32,
    pub window_width: Vector2<u32>,
}

pub trait DynamicToolUi {
    fn label(&mut self, contents: &str);
    fn dropdown(
        &mut self,
        label: &str,
        current: usize,
        values_fn: Box<dyn FnOnce() -> Vec<(usize, String)>>,
    ) -> usize;
    fn button(&mut self, label: &str) -> bool;
    fn value_float(&mut self, label: &str, current: f32) -> f32 {
        self.value_float_ranged(label, current, f32::MIN..=f32::MAX)
    }

    fn value_float_ranged(&mut self, label: &str, current: f32, range: RangeInclusive<f32>) -> f32;

    fn textbox_float_ranged(
        &mut self,
        label: &str,
        current: f32,
        range: RangeInclusive<f32>,
    ) -> f32;
    fn vec2_ranged(
        &mut self,
        label: &str,
        value: &mut cgmath::Vector2<f32>,
        x_min: RangeInclusive<f32>,
        y_min: RangeInclusive<f32>,
    );
}

pub mod dynamic_tool_ui_helpers {
    use strum::IntoEnumIterator;

    use super::DynamicToolUi;

    pub fn dropdown<T: Copy + ToString + IntoEnumIterator + From<usize>>(
        ui: &mut dyn DynamicToolUi,
        label: &str,
        current: T,
    ) -> T
    where
        usize: From<T>,
    {
        let selection = ui.dropdown(
            label,
            usize::from(current),
            Box::new(|| T::iter().map(|v| (usize::from(v), v.to_string())).collect()),
        );
        T::from(selection)
    }
}

pub trait Tool {
    fn name(&self) -> &'static str;
    fn on_selected(&mut self, _context: &mut EditorContext) -> Option<Box<dyn EditorCommand>> {
        None
    }
    fn on_deselected(&mut self, _context: &mut EditorContext) -> Option<Box<dyn EditorCommand>> {
        None
    }
    fn on_pointer_click(
        &mut self,
        _pointer_click: PointerEvent,
        _context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        None
    }
    fn on_pointer_move(
        &mut self,
        _pointer_motion: PointerEvent,
        _context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        None
    }

    fn ui(&mut self, _ui: &mut dyn DynamicToolUi, _context: &mut EditorContext) {}

    fn on_pointer_release(
        &mut self,
        _pointer_release: PointerEvent,
        _context: &mut EditorContext,
    ) -> Option<Box<dyn EditorCommand>> {
        None
    }

    fn draw(&self, _renderer: &mut Renderer) {}
}
