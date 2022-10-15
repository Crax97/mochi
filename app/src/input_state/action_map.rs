use std::{hash::Hash};

use egui::epaint::ahash::HashMap;

use super::{
    key::{Key, ModifierSet},
    InputState,
};

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub enum ActionState {
    Pressed,
    Released,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct KeyBinding {
    pub key: (Key, ActionState),
    pub modifiers: ModifierSet,
}

impl From<(Key, ActionState)> for KeyBinding {
    fn from(key: (Key, ActionState)) -> Self {
        KeyBinding {
            key,
            modifiers: ModifierSet::default(),
        }
    }
}

pub struct ActionMap<T> {
    keybindings_to_action_name: HashMap<KeyBinding, T>,
}

impl<T> Default for ActionMap<T> {
    fn default() -> Self {
        Self {
            keybindings_to_action_name: Default::default(),
        }
    }
}

impl<T: Clone> ActionMap<T> {
    pub fn update(&mut self, input_state: &InputState) -> Vec<T> {
        self.keybindings_to_action_name
            .iter()
            .filter(|(keybinding, _)| self.get_keybinding_state(keybinding, input_state))
            .map(|(_, a)| a.clone())
            .collect()
    }

    fn get_keybinding_state(&self, binding: &KeyBinding, input_state: &InputState) -> bool {
        let modifiers = input_state.current_modifiers() == &binding.modifiers;
        if !modifiers {
            return false;
        }
        match binding.key.1 {
            ActionState::Pressed => input_state.is_key_just_pressed(binding.key.0),
            ActionState::Released => input_state.is_key_just_released(binding.key.0),
        }
    }

    pub fn add_action_binding<K: Into<KeyBinding>, S: Into<T>>(
        &mut self,
        keybinding: K,
        action_name: S,
    ) {
        let keybinding = keybinding.into();
        let result = self
            .keybindings_to_action_name
            .insert(keybinding, action_name.into());
        debug_assert!(
            result.is_none(),
            "Only one keybinding per action is allowed!"
        );
    }
}
