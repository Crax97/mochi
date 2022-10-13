pub mod action_map;
pub mod key;

pub use action_map::*;
pub use key::*;

use std::collections::HashMap;

use cgmath::{Point2, Vector2};
use strum::EnumCount;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyboardInput, ModifiersState, MouseButton, MouseScrollDelta},
};

use self::key::{Key, ModifierSet};

#[derive(Debug)]
pub struct InputState {
    current_cursor_position: PhysicalPosition<f32>,
    last_update_cursor_position: PhysicalPosition<f32>,
    current_pointer_pressure: f32,
    window_size: PhysicalSize<u32>,
    current_wheel_delta: f32,

    pointer_button_state: HashMap<MouseButton, ElementState>,
    last_button_state: HashMap<MouseButton, ElementState>,
    key_states: [bool; Key::COUNT],
    last_key_states: [bool; Key::COUNT],

    current_modifiers: ModifierSet,
    last_modifiers: ModifierSet,
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
            key_states: [false; Key::COUNT],
            last_key_states: [false; Key::COUNT],
            current_modifiers: ModifierSet::default(),
            last_modifiers: ModifierSet::default(),
        }
    }
}

impl InputState {
    pub(crate) fn update(&mut self, event: &winit::event::Event<()>) {
        self.last_button_state = self.pointer_button_state.clone();
        self.last_key_states = self.key_states.clone();
        self.last_modifiers = self.current_modifiers.clone();
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
                winit::event::WindowEvent::KeyboardInput { input, .. } => {
                    self.update_keyboard_state(input);
                }
                winit::event::WindowEvent::ModifiersChanged(modifiers) => {
                    self.update_modifiers_state(modifiers);
                }
                winit::event::WindowEvent::CursorMoved { position, .. } => {
                    self.last_update_cursor_position = self.current_cursor_position;
                    let mouse_position = position.cast::<f32>();
                    self.current_cursor_position = PhysicalPosition {
                        x: mouse_position.x,
                        y: self.window_size.height as f32 - mouse_position.y,
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
                    self.set_cursor_button_state(*button, *state);
                }
                winit::event::WindowEvent::TouchpadPressure { pressure, .. } => {
                    self.current_pointer_pressure = *pressure
                }
                winit::event::WindowEvent::Touch(touch) => {
                    let winit::event::Touch {
                        phase,
                        location,
                        force,
                        ..
                    } = touch;

                    if let Some(force) = force {
                        self.current_pointer_pressure = match force {
                            winit::event::Force::Calibrated {
                                force,
                                max_possible_force,
                                ..
                            } => force / max_possible_force,
                            winit::event::Force::Normalized(force) => *force,
                        } as f32;
                    }
                    match phase {
                        winit::event::TouchPhase::Started => {
                            self.set_cursor_button_state(
                                winit::event::MouseButton::Left,
                                ElementState::Pressed,
                            );
                        }
                        winit::event::TouchPhase::Moved => {
                            self.current_cursor_position = location.cast::<f32>();
                        }
                        winit::event::TouchPhase::Ended | winit::event::TouchPhase::Cancelled => {
                            self.set_cursor_button_state(
                                winit::event::MouseButton::Left,
                                ElementState::Released,
                            );
                        }
                    }
                }
                _ => {}
            },
            winit::event::Event::DeviceEvent { .. } => {}
            _ => {}
        }
    }

    fn set_cursor_button_state(&mut self, button: MouseButton, state: ElementState) {
        self.pointer_button_state
            .entry(button)
            .and_modify(|s| *s = state)
            .or_insert(state);
    }

    pub(crate) fn mouse_position(&self) -> Point2<f32> {
        Point2 {
            x: self.current_cursor_position.x,
            y: self.current_cursor_position.y,
        }
    }
    pub(crate) fn last_position(&self) -> PhysicalPosition<f32> {
        self.last_update_cursor_position
    }
    pub(crate) fn normalized_mouse_position(&self) -> Point2<f32> {
        Point2 {
            x: (self.current_cursor_position.x / self.window_size.width as f32) * 2.0 - 1.0,
            y: ((self.current_cursor_position.y / self.window_size.height as f32) * 2.0 - 1.0),
        }
    }
    pub(crate) fn normalized_last_mouse_position(&self) -> Point2<f32> {
        Point2 {
            x: (self.last_update_cursor_position.x / self.window_size.width as f32) * 2.0 - 1.0,
            y: ((self.last_update_cursor_position.y / self.window_size.height as f32) * 2.0 - 1.0),
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

    pub(crate) fn current_pointer_pressure(&self) -> f32 {
        self.current_pointer_pressure
    }

    pub(crate) fn window_size(&self) -> Vector2<u32> {
        Vector2 {
            x: self.window_size.width,
            y: self.window_size.height,
        }
    }

    pub(crate) fn is_key_just_pressed(&self, key: Key) -> bool {
        self.key_states[key as usize] && !self.last_key_states[key as usize]
    }

    pub(crate) fn is_key_just_released(&self, key: Key) -> bool {
        !self.key_states[key as usize] && self.last_key_states[key as usize]
    }

    pub(crate) fn is_key_pressed(&self, key: Key) -> bool {
        self.key_states[key as usize]
    }

    pub(crate) fn is_key_released(&self, key: Key) -> bool {
        !self.is_key_pressed(key)
    }

    pub(crate) fn current_modifiers(&self) -> &ModifierSet {
        &self.current_modifiers
    }

    fn update_keyboard_state(&mut self, input: &KeyboardInput) {
        if let Some(virtual_key) = input.virtual_keycode {
            let key: Key = virtual_key.into();
            self.key_states[key as usize] = match input.state {
                ElementState::Pressed => true,
                ElementState::Released => false,
            }
        }
    }

    fn update_modifiers_state(&mut self, modifiers: &ModifiersState) {
        self.current_modifiers = modifiers.bits().into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::event::*;

    #[test]
    pub fn test_key_events() {
        let mut input_state = InputState::default();

        input_state.update(&Event::WindowEvent {
            window_id: unsafe { winit::window::WindowId::dummy() },
            event: WindowEvent::KeyboardInput {
                device_id: unsafe { DeviceId::dummy() },
                input: KeyboardInput {
                    scancode: 0,
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::W),
                    modifiers: ModifiersState::empty(),
                },
                is_synthetic: false,
            },
        });

        assert!(input_state.is_key_just_pressed(Key::W));
        assert!(input_state.is_key_pressed(Key::W));
        assert!(input_state.is_key_released(Key::A));

        input_state.update(&Event::WindowEvent {
            window_id: unsafe { winit::window::WindowId::dummy() },
            event: WindowEvent::KeyboardInput {
                device_id: unsafe { DeviceId::dummy() },
                input: KeyboardInput {
                    scancode: 0,
                    state: ElementState::Released,
                    virtual_keycode: Some(VirtualKeyCode::W),
                    modifiers: ModifiersState::empty(),
                },
                is_synthetic: false,
            },
        });
        assert!(input_state.is_key_just_released(Key::W));
        assert!(input_state.is_key_released(Key::W));
        assert!(input_state.is_key_released(Key::A));
    }

    #[test]
    pub fn test_modifiers() {
        let mut input_state = InputState::default();

        input_state.update(&Event::WindowEvent {
            window_id: unsafe { winit::window::WindowId::dummy() },
            event: WindowEvent::ModifiersChanged(ModifiersState::from_bits_truncate(
                ModifiersState::SHIFT.bits() | ModifiersState::CTRL.bits(),
            )),
        });

        assert_eq!(
            input_state.current_modifiers(),
            &ModifierSet::new(true, false, true, false)
        );
        assert_ne!(
            input_state.current_modifiers(),
            &ModifierSet::new(true, true, true, false)
        );

        input_state.update(&Event::WindowEvent {
            window_id: unsafe { winit::window::WindowId::dummy() },
            event: WindowEvent::ModifiersChanged(ModifiersState::empty()),
        });

        assert_eq!(
            input_state.current_modifiers(),
            &ModifierSet::new(false, false, false, false)
        );
        assert_ne!(
            input_state.current_modifiers(),
            &ModifierSet::new(true, false, false, false)
        );
    }
}
