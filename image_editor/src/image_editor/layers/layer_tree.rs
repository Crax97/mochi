use std::collections::HashMap;

use framework::{framework::TextureId, renderer::renderer::Renderer, Framework};

use super::{Layer, LayerId};

#[derive(Clone, PartialEq, PartialOrd, Eq, Hash)]
pub(crate) enum LayerItem {
    SingleLayer(LayerId),
    Group(Vec<LayerItem>),
}

pub(crate) trait LayerRenderingStrategy {
    fn new(framework: &mut Framework) -> Self
    where
        Self: Sized;
    fn on_new_layer(&mut self, layer: &Layer, framework: &mut Framework);
    fn on_layer_removed(&mut self, layer: &Layer);
    fn update(&mut self, layers: &Vec<LayerItem>, framework: &mut Framework);
    fn render(
        &mut self,
        layers: &Vec<LayerItem>,
        framework: &mut Framework,
        renderer: &mut Renderer,
    ) -> TextureId;
}

pub(crate) struct LayerTree<T: LayerRenderingStrategy> {
    layers: HashMap<LayerId, Layer>,
    items: Vec<LayerItem>,
    current_layer_id: Option<LayerId>,
    rendering_strategy: T,
}

impl<T: LayerRenderingStrategy> LayerTree<T> {
    pub fn new(framework: &mut Framework) -> Self {
        Self {
            layers: HashMap::new(),
            items: Vec::new(),
            current_layer_id: None,
            rendering_strategy: T::new(framework),
        }
    }

    pub fn current_layer(&self) -> Option<&Layer> {
        if let Some(id) = &self.current_layer_id {
            self.layers.get(id)
        } else {
            None
        }
    }
    pub fn select_layer(&mut self, id: LayerId) {
        assert!(self.layers.contains_key(&id));
        self.current_layer_id = Some(id);
    }

    fn add_layer_impl(
        new_layer_id: LayerId,
        current_layer_id: &LayerId,
        existing_layers: &mut Vec<LayerItem>,
        framework: &mut Framework,
    ) {
        let mut place = None;
        for (layer_idx, mut item) in existing_layers.iter_mut().enumerate() {
            match &mut item {
                LayerItem::SingleLayer(id) => {
                    if id == current_layer_id {
                        place = Some(layer_idx);
                        break;
                    }
                }
                LayerItem::Group(items) => {
                    Self::add_layer_impl(new_layer_id, current_layer_id, items, framework)
                }
            }
        }
        if let Some(id) = place {
            existing_layers.insert(id, LayerItem::SingleLayer(new_layer_id));
        }
    }
    pub fn add_layer(&mut self, layer: Layer, framework: &mut Framework) {
        if let Some(current_id) = &self.current_layer_id {
            Self::add_layer_impl(layer.id().clone(), current_id, &mut self.items, framework);
        } else {
            self.items.push(LayerItem::SingleLayer(layer.id().clone()));
        }
        self.rendering_strategy.on_new_layer(&layer, framework);
        self.layers.insert(layer.id().clone(), layer);
    }

    pub fn get_layer(&self, layer_index: &LayerId) -> &Layer {
        self.layers.get(layer_index).unwrap()
    }

    fn find_below_impl(
        target_layer_id: &LayerId,
        existing_layers: &Vec<LayerItem>,
    ) -> Option<LayerId> {
        let mut spot = None;
        let iter = existing_layers.iter().enumerate();
        for (idx, item) in iter {
            match &item {
                LayerItem::SingleLayer(id) => {
                    if id == target_layer_id {
                        spot = Some(idx);
                        break;
                    }
                }
                LayerItem::Group(items) => {
                    return Self::find_below_impl(target_layer_id, items);
                }
            }
        }
        if let Some(idx) = spot {
            return if let Some(LayerItem::SingleLayer(l)) = existing_layers.get(idx + 1) {
                Some(l.clone())
            } else {
                None
            };
        } else {
            None
        }
    }
    pub fn find_below(&mut self, layer: &LayerId) -> Option<LayerId> {
        Self::find_below_impl(layer, &self.items)
    }

    fn find_above_impl(
        target_layer_id: &LayerId,
        existing_layers: &Vec<LayerItem>,
    ) -> Option<LayerId> {
        let mut prev = None;
        let iter = existing_layers.iter();
        for item in iter {
            match &item {
                LayerItem::SingleLayer(id) => {
                    if id == target_layer_id {
                        return prev;
                    } else {
                        prev = Some(id.clone());
                    }
                }
                LayerItem::Group(items) => {
                    return Self::find_above_impl(target_layer_id, items);
                }
            }
        }
        None
    }
    pub fn find_above(&mut self, layer: &LayerId) -> Option<LayerId> {
        Self::find_above_impl(layer, &self.items)
    }

    fn remove_layer_impl(layer_to_remove_id: &LayerId, existing_layers: &mut Vec<LayerItem>) {
        for (layer_idx, mut item) in existing_layers.iter_mut().enumerate() {
            match &mut item {
                LayerItem::SingleLayer(id) => {
                    if id == layer_to_remove_id {
                        existing_layers.remove(layer_idx);
                        return;
                    }
                }
                LayerItem::Group(items) => Self::remove_layer_impl(layer_to_remove_id, items),
            }
        }
    }
    pub fn remove_layer(&mut self, layer_id: LayerId) {
        Self::remove_layer_impl(&layer_id, &mut self.items);
        let layer = self
            .layers
            .remove(&layer_id)
            .expect("LayerTree: layer not found");
        self.rendering_strategy.on_layer_removed(&layer);

        if self.current_layer_id.map_or(false, |id| id == layer_id) {
            self.current_layer_id = self.find_below(&layer_id);
        }
    }

    fn for_each_layer_impl<F: FnMut(&Layer)>(
        f: &mut F,
        existing_items: &Vec<LayerItem>,
        existing_layers: &HashMap<LayerId, Layer>,
    ) {
        for item in existing_items.iter() {
            match &item {
                LayerItem::SingleLayer(id) => {
                    let layer = existing_layers.get(id).unwrap();
                    f(layer);
                }
                LayerItem::Group(ref items) => {
                    Self::for_each_layer_impl(f, items, existing_layers);
                }
            }
        }
    }
    pub fn for_each_layer<F: FnMut(&Layer)>(&self, mut f: F) {
        Self::for_each_layer_impl(&mut f, &self.items, &self.layers);
    }

    pub fn update(&mut self, framework: &mut Framework) {
        self.rendering_strategy.update(&self.items, framework);
    }

    pub fn render(&mut self, framework: &mut Framework, renderer: &mut Renderer) -> TextureId {
        self.rendering_strategy
            .render(&self.items, framework, renderer)
    }
}

pub struct CanvasRenderingStrategy {}

impl LayerRenderingStrategy for CanvasRenderingStrategy {
    fn new(framework: &mut Framework) -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn on_new_layer(&mut self, layer: &Layer, framework: &mut Framework) {
        todo!()
    }

    fn on_layer_removed(&mut self, layer: &Layer) {
        todo!()
    }

    fn update(&mut self, layers: &Vec<LayerItem>, framework: &mut Framework) {
        todo!()
    }

    fn render(
        &mut self,
        layers: &Vec<LayerItem>,
        framework: &mut Framework,
        renderer: &mut Renderer,
    ) -> TextureId {
        todo!()
    }
}
