use std::collections::HashMap;

use cgmath::{point2, vec2, Vector2};
use framework::TypedBuffer;
use wgpu::BindGroup;

use crate::layers::{BitmapLayerConfiguration, LayerCreationInfo, LayerTree};

use super::layers::{BitmapLayer, Layer, LayerIndex, RootLayer};

pub struct Document<'framework> {
    pub width: u32,
    pub height: u32,
    pub layers: HashMap<LayerIndex, Layer<'framework>>,
    pub tree_root: RootLayer,
    pub final_layer: Layer<'framework>,

    pub current_layer_index: LayerIndex,
}

impl<'l> Document<'l> {
    pub fn outer_size(&self) -> Vector2<f32> {
        match self.final_layer.layer_type {
            crate::layers::LayerType::Bitmap(ref bm) => bm.size().clone(),
        }
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

    pub fn get_layer_mut(&mut self, layer_index: &LayerIndex) -> &mut Layer<'l> {
        self.layers
            .get_mut(&layer_index)
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

    pub(crate) fn final_layer_texture_view(&self) -> &wgpu::TextureView {
        match self.final_layer.layer_type {
            crate::layers::LayerType::Bitmap(ref bm) => bm.texture_view(),
        }
    }

    pub(crate) fn final_texture(&self) -> &wgpu::Texture {
        match self.final_layer.layer_type {
            crate::layers::LayerType::Bitmap(ref bm) => bm.texture(),
        }
    }

    pub(crate) fn final_bind_group(&self) -> &BindGroup {
        match self.final_layer.layer_type {
            crate::layers::LayerType::Bitmap(ref bm) => bm.bind_group(),
        }
    }
}
