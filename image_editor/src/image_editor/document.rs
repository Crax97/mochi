use std::collections::HashMap;

use cgmath::Vector2;

use super::layers::{BitmapLayer, Layer, LayerIndex, RootLayer};

pub struct Document<'framework> {
    pub layers: HashMap<LayerIndex, Layer<'framework>>,
    pub tree_root: RootLayer,
    pub final_layer: BitmapLayer,

    pub current_layer_index: LayerIndex,
}

impl Document<'_> {
    pub fn outer_size(&self) -> Vector2<f32> {
        self.final_layer.size()
    }

    pub fn current_layer(&self) -> &Layer {
        self.get_layer(&self.current_layer_index)
    }

    pub fn select_layer(&mut self, new_current_layer: LayerIndex) {
        assert!(self.layers.contains_key(&new_current_layer));
        self.current_layer_index = new_current_layer;
    }

    pub fn get_layer(&self, layer_index: &LayerIndex) -> &Layer {
        self.layers
            .get(&layer_index)
            .expect("Invalid layer index passed to document!")
    }
}
