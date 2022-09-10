use std::collections::HashMap;

use cgmath::{point2, vec2, Vector2};
use framework::TypedBuffer;

use crate::layers::{BitmapLayerConfiguration, LayerCreationInfo, LayerTree};

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
    pub(crate) fn delete_layer(&mut self, layer_idx: LayerIndex) {
        if self.layers.len() == 1 {
            return;
        }
        if self.current_layer_index == layer_idx {
            let new_layer = self
                .layers
                .keys()
                .find(|layer_id| **layer_id != layer_idx)
                .unwrap();
            self.select_layer(new_layer.clone());
        }
        self.layers.remove(&layer_idx);
        let mut erase_which = 0usize;
        for (i, layer) in self.tree_root.0.iter().enumerate() {
            match layer {
                &LayerTree::SingleLayer(idx) if idx == layer_idx => {
                    erase_which = i;
                }
                LayerTree::Group(_) => todo!(),
                _ => {}
            }
        }
        self.tree_root.0.remove(erase_which);
    }
}
