use std::collections::HashMap;

use cgmath::{Point2, Vector2};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, MouseScrollDelta},
};

#[derive(Debug)]
pub(crate) struct InputState {
    current_cursor_position: PhysicalPosition<f32>,
    last_update_cursor_position: PhysicalPosition<f32>,
    current_pointer_pressure: f32,
    window_size: PhysicalSize<u32>,
    current_wheel_delta: f32,

    pointer_button_state: HashMap<MouseButton, ElementState>,
    last_button_state: HashMap<MouseButton, ElementState>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            current_cursor_position: Default::default(),
            last_update_cursor_position: Default::default(),
            current_pointer_pressure: Default::default(),
            window_size: Default::default(),
            current_wheel_delta: 0.0,
            pointer_button_state: Default::default(),
            last_button_state: Default::default(),
        }
    }
}

impl InputState {
    pub(crate) fn update(&mut self, event: &winit::event::Event<()>) {
        self.last_button_state = self.pointer_button_state.clone();
        self.current_wheel_delta = 0.0;
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::Resized(new_size) => self.window_size = *new_size,
                winit::event::WindowEvent::Moved(_) => {}
                winit::event::WindowEvent::CloseRequested => {}
                winit::event::WindowEvent::DroppedFile(_) => {}
                winit::event::WindowEvent::HoveredFile(_) => {}
                winit::event::WindowEvent::HoveredFileCancelled => {}
                winit::event::WindowEvent::ReceivedCharacter(_) => {}
                winit::event::WindowEvent::KeyboardInput {
                    device_id,
                    input,
                    is_synthetic,
                } => {}
                winit::event::WindowEvent::ModifiersChanged(_) => {}
                winit::event::WindowEvent::CursorMoved { position, .. } => {
                    self.last_update_cursor_position = self.current_cursor_position;
                    let mouse_position = position.cast::<f32>();
                    self.current_cursor_position = PhysicalPosition {
                        x: mouse_position.x,
                        y: mouse_position.y,
                    };
                }
                winit::event::WindowEvent::MouseWheel { delta, .. } => {
                    self.current_wheel_delta = match delta {
                        MouseScrollDelta::LineDelta(_, y) => *y,
                        MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                    } / self.window_size.height as f32
                        * 0.5;
                }
                winit::event::WindowEvent::MouseInput { state, button, .. } => {
                    self.pointer_button_state
                        .entry(*button)
                        .and_modify(|s| *s = *state)
                        .or_insert(*state);
                }
                winit::event::WindowEvent::TouchpadPressure { pressure, .. } => {
                    self.current_pointer_pressure = *pressure
                }
                winit::event::WindowEvent::Touch(_) => {}
                _ => {}
            },
            winit::event::Event::DeviceEvent { device_id, event } => {}
            _ => {}
        }
    }

    pub(crate) fn mouse_position(&self) -> PhysicalPosition<f32> {
        self.current_cursor_position
    }
    pub(crate) fn last_position(&self) -> PhysicalPosition<f32> {
        self.last_update_cursor_position
    }
    pub(crate) fn normalized_mouse_position(&self) -> Point2<f32> {
        Point2 {
            x: (self.current_cursor_position.x / self.window_size.width as f32) * 2.0 - 1.0,
            y: -((self.current_cursor_position.y / self.window_size.height as f32) * 2.0 - 1.0),
        }
    }
    pub(crate) fn normalized_last_mouse_position(&self) -> Point2<f32> {
        Point2 {
            x: (self.last_update_cursor_position.x / self.window_size.width as f32) * 2.0 - 1.0,
            y: -((self.last_update_cursor_position.y / self.window_size.height as f32) * 2.0 - 1.0),
        }
    }
    pub(crate) fn mouse_delta(&self) -> Vector2<f32> {
        Vector2 {
            x: self.current_cursor_position.x - self.last_update_cursor_position.x,
            y: self.current_cursor_position.y - self.last_update_cursor_position.y,
        }
    }
    pub(crate) fn normalized_mouse_delta(&self) -> Vector2<f32> {
        self.normalized_mouse_position() - self.normalized_last_mouse_position()
    }

    pub(crate) fn is_mouse_button_just_pressed(&self, button: MouseButton) -> bool {
        match (
            self.pointer_button_state.get(&button),
            self.last_button_state.get(&button),
        ) {
            (Some(now), Some(before)) => {
                now == &ElementState::Pressed && before == &ElementState::Released
            }
            (Some(now), None) => now == &ElementState::Pressed,
            _ => false,
        }
    }
    pub(crate) fn is_mouse_button_just_released(&self, button: MouseButton) -> bool {
        match (
            self.pointer_button_state.get(&button),
            self.last_button_state.get(&button),
        ) {
            (Some(now), Some(before)) => {
                now == &ElementState::Released && before == &ElementState::Pressed
            }
            (Some(now), None) => now == &ElementState::Released,
            _ => false,
        }
    }
    pub(crate) fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.pointer_button_state
            .get(&button)
            .map_or(false, |btn| btn == &ElementState::Pressed)
    }
    pub(crate) fn is_mouse_button_released(&self, button: MouseButton) -> bool {
        self.pointer_button_state
            .get(&button)
            .map_or(false, |btn| btn == &ElementState::Released)
    }

    pub(crate) fn mouse_wheel_delta(&self) -> f32 {
        self.current_wheel_delta
    }
}
