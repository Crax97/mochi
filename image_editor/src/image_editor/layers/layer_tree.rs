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
    fn on_layer_removed(&mut self, layer: &Layer, framework: &mut Framework);
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

    pub fn add_layer(&mut self, layer: Layer) -> LayerId {
        todo!()
    }

    pub fn get_layer(&self, layer_index: &LayerId) -> &Layer {
        todo!()
    }

    pub fn find_below(&mut self, layer: &LayerId) -> Option<LayerId> {
        todo!()
    }

    pub fn find_above(&mut self, layer: &LayerId) -> Option<LayerId> {
        todo!()
    }

    pub fn remove_layer(&mut self, layer_id: LayerId) -> Option<LayerId> {
        todo!()
    }

    pub fn for_each_layer<F: FnMut(&Layer)>(&self, f: F) {
        todo!()
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
        todo!()
    }

    fn on_new_layer(&mut self, layer: &Layer, framework: &mut Framework) {
        todo!()
    }

    fn on_layer_removed(&mut self, layer: &Layer, framework: &mut Framework) {
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
