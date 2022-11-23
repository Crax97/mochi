use std::collections::HashMap;

use framework::{
    framework::{BufferId, TextureId},
    renderer::{
        draw_command::{
            BindableResource, DrawCommand,
            DrawMode::{self, Single},
            OptionalDrawData,
            PrimitiveType::{self, Texture2D},
        },
        renderer::Renderer,
    },
    BufferConfiguration, Camera2d, Framework, RgbaTexture2D, Texture, TextureConfiguration,
    TextureUsage, Transform2d,
};

use crate::{
    blend_settings::{BlendMode, BlendSettings, BlendSettingsUniform},
    document::DocumentCreationInfo,
    image_editor::ab_render_target::ABRenderTarget,
};

use super::{Layer, LayerId};

#[derive(Clone, PartialEq, PartialOrd, Eq, Hash)]
pub(crate) enum LayerItem {
    SingleLayer(LayerId),
    Group(Vec<LayerItem>, LayerId),
}

pub(crate) trait LayerRenderingStrategy {
    fn new(framework: &mut Framework, document_info: &DocumentCreationInfo) -> Self
    where
        Self: Sized;
    fn on_new_layer(&mut self, layer: &Layer, framework: &mut Framework);
    fn on_layer_removed(&mut self, layer: &Layer);
    fn update(&mut self, layers: &HashMap<LayerId, Layer>, framework: &mut Framework);
    fn update_canvases(
        &mut self,
        layers: &Vec<LayerItem>,
        layers: &HashMap<LayerId, Layer>,
        framework: &mut Framework,
        renderer: &mut Renderer,
    );

    fn composite_layer_on_target(
        &self,
        id: &LayerId,
        back: &TextureId,
        resulting_texture: &TextureId,
        renderer: &mut Renderer,
        framework: &mut Framework,
    );
}

pub(crate) struct LayerTree<T: LayerRenderingStrategy> {
    layers: HashMap<LayerId, Layer>,
    items: Vec<LayerItem>,
    current_layer_id: Option<LayerId>,
    rendering_strategy: T,
}

impl<T: LayerRenderingStrategy> LayerTree<T> {
    pub fn new(framework: &mut Framework, document_info: &DocumentCreationInfo) -> Self {
        Self {
            layers: HashMap::new(),
            items: Vec::new(),
            current_layer_id: None,
            rendering_strategy: T::new(framework, document_info),
        }
    }

    pub fn current_layer(&self) -> Option<&Layer> {
        if let Some(id) = &self.current_layer_id {
            self.layers.get(id)
        } else {
            None
        }
    }
    pub fn current_layer_id(&self) -> Option<&LayerId> {
        self.current_layer_id.as_ref()
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
                LayerItem::Group(items, ..) => {
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
                LayerItem::Group(.., id) => {
                    return Some(id.clone());
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
                LayerItem::Group(.., id) => {
                    return Some(id.clone());
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
                LayerItem::Group(children, id) => {
                    todo!()
                }
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
                LayerItem::Group(ref items, id) => {
                    f(existing_layers.get(id).unwrap());
                    Self::for_each_layer_impl(f, items, existing_layers);
                }
            }
        }
    }
    pub fn for_each_layer<F: FnMut(&Layer)>(&self, mut f: F) {
        Self::for_each_layer_impl(&mut f, &self.items, &self.layers);
    }

    pub fn update(&mut self, framework: &mut Framework) {
        self.rendering_strategy.update(&self.layers, framework);
    }

    pub fn render(&mut self, framework: &mut Framework, renderer: &mut Renderer) {
        self.rendering_strategy
            .update_canvases(&self.items, &self.layers, framework, renderer)
    }

    pub fn composite_final_image(
        &mut self,
        width: u32,
        height: u32,
        renderer: &mut Renderer,
        framework: &mut Framework,
    ) -> TextureId {
        Self::composite_final_image_impl(
            &self.items,
            &self.rendering_strategy,
            width,
            height,
            renderer,
            framework,
        )
    }

    fn composite_final_image_impl(
        items: &Vec<LayerItem>,
        strategy: &T,
        width: u32,
        height: u32,
        renderer: &mut Renderer,
        framework: &mut Framework,
    ) -> TextureId {
        let mut ab_render_target = ABRenderTarget::new(width, height, framework);
        for item in items {
            match item {
                LayerItem::SingleLayer(id) => {
                    ab_render_target.run_render_loop(|result, back| {
                        strategy.composite_layer_on_target(id, back, &result, renderer, framework);
                    });
                }
                LayerItem::Group(items, group_layer_id) => {
                    let rendered_group = Self::composite_final_image_impl(
                        items, strategy, width, height, renderer, framework,
                    );

                    ab_render_target.run_render_loop(|result, back| {
                        strategy.composite_layer_on_target(
                            group_layer_id,
                            back,
                            &result,
                            renderer,
                            framework,
                        );
                    });
                }
            }
        }
        ab_render_target.result().clone()
    }
}

pub(crate) struct LayerCanvasData {
    pub(crate) canvas: TextureId,
    pub(crate) settings_buffer: BufferId,
}

pub struct CanvasRenderingStrategy {
    layer_datas: HashMap<LayerId, LayerCanvasData>,
    document_width: u32,
    document_height: u32,
}

impl LayerRenderingStrategy for CanvasRenderingStrategy {
    fn new(framework: &mut Framework, document_info: &DocumentCreationInfo) -> Self
    where
        Self: Sized,
    {
        Self {
            layer_datas: HashMap::new(),
            document_height: document_info.height,
            document_width: document_info.width,
        }
    }

    fn on_new_layer(&mut self, layer: &Layer, framework: &mut Framework) {
        let canvas = framework.allocate_texture2d(
            RgbaTexture2D::empty((self.document_width, self.document_height)),
            TextureConfiguration {
                label: Some(format!("Canvas texture for layer {:?}", layer.id()).as_str()),
                usage: TextureUsage::RWRT,
                mip_count: None,
            },
        );
        let settings_buffer =
            framework.allocate_typed_buffer(BufferConfiguration::<BlendSettingsUniform> {
                initial_setup: framework::buffer::BufferInitialSetup::Count(1),
                buffer_type: framework::BufferType::Uniform,
                gpu_copy_dest: true,
                gpu_copy_source: false,
                cpu_copy_dest: false,
                cpu_copy_source: false,
            });
        self.layer_datas.insert(
            layer.id().clone(),
            LayerCanvasData {
                canvas,
                settings_buffer,
            },
        );
    }

    fn on_layer_removed(&mut self, layer: &Layer) {
        self.layer_datas
            .remove(&layer.id())
            .expect("CanvasRenderingStrategy: layer not found");
    }

    fn update(&mut self, layers: &HashMap<LayerId, Layer>, framework: &mut Framework) {
        Self::update_impl(layers, &self.layer_datas, framework)
    }

    fn update_canvases(
        &mut self,
        items: &Vec<LayerItem>,
        layers: &HashMap<LayerId, Layer>,
        framework: &mut Framework,
        renderer: &mut Renderer,
    ) {
        Self::render_impl(
            self.document_width,
            self.document_height,
            items,
            layers,
            &self.layer_datas,
            framework,
            renderer,
        );
    }
    fn composite_layer_on_target(
        &self,
        id: &LayerId,
        back: &TextureId,
        canvas: &TextureId,
        renderer: &mut Renderer,
        framework: &mut Framework,
    ) {
        let source = self.layer_data(id);
        renderer.begin(&Camera2d::default(), None, framework);
        renderer.draw(DrawCommand {
            primitives: PrimitiveType::Texture2D {
                texture_id: source.canvas.clone(),
                instances: vec![Transform2d::default()],
                flip_uv_y: false,
                multiply_color: wgpu::Color::WHITE,
            },
            draw_mode: DrawMode::Single,
            additional_data: OptionalDrawData {
                additional_vertex_buffers: vec![],
                additional_bindable_resource: vec![
                    BindableResource::Texture(back.clone()),
                    BindableResource::UniformBuffer(source.settings_buffer.clone()),
                ],
                shader: Some(crate::global_selection_data().blended_shader.clone()),
            },
        });
        renderer.end(&canvas, None, framework);
    }
}

impl CanvasRenderingStrategy {
    pub(crate) fn layer_data(&self, id: &LayerId) -> &LayerCanvasData {
        self.layer_datas
            .get(id)
            .expect("CanvasRenderingStrategy::layer_data(): no layer with id")
    }

    fn update_impl(
        layers: &HashMap<LayerId, Layer>,
        datas: &HashMap<LayerId, LayerCanvasData>,
        framework: &mut Framework,
    ) {
        for layer in layers.values() {
            if layer.needs_settings_update() {
                let data = datas
                    .get(&layer.id())
                    .expect("CanvasRenderingStrategy: layer not found");
                framework.buffer_write_sync(
                    &data.settings_buffer,
                    vec![BlendSettingsUniform::from(BlendSettings {
                        blend_mode: layer.settings().blend_mode,
                    })],
                );
            }
        }
    }

    fn make_camera_for_layer(layer: &Layer) -> Camera2d {
        let size = layer.bounds().extents * 0.5;
        Camera2d::new(-0.01, 1000.0, [-size.x, size.x, size.y, -size.y])
    }

    fn render_image(
        image_texture: &TextureId,
        owning_layer: &Layer,
        target: &TextureId,
        transform: &Transform2d,
        framework: &mut Framework,
        renderer: &mut Renderer,
    ) {
        renderer.begin(
            &Self::make_camera_for_layer(owning_layer),
            Some(wgpu::Color::TRANSPARENT),
            framework,
        );
        renderer.draw(DrawCommand {
            primitives: Texture2D {
                texture_id: image_texture.clone(),
                instances: vec![transform.clone()],
                flip_uv_y: false,
                multiply_color: wgpu::Color::WHITE,
            },
            draw_mode: Single,
            additional_data: OptionalDrawData::default(),
        });
        renderer.end(target, None, framework);
    }
    fn render_layer(
        layer: &Layer,
        target: &TextureId,
        framework: &mut Framework,
        renderer: &mut Renderer,
    ) {
        match &layer.layer_type {
            super::LayerType::Image { texture, .. } => Self::render_image(
                texture,
                layer,
                target,
                &layer.pixel_transform(),
                framework,
                renderer,
            ),
            super::LayerType::Group => {
                unreachable!() // LayerType Group aren't supposed to be rendered directly
            }
        }
    }

    fn render_impl(
        width: u32,
        height: u32,
        items: &Vec<LayerItem>,
        layers: &HashMap<LayerId, Layer>,
        datas: &HashMap<LayerId, LayerCanvasData>,
        framework: &mut Framework,
        renderer: &mut Renderer,
    ) {
        for item in items {
            match item {
                LayerItem::SingleLayer(id) => {
                    let layer = layers
                        .get(id)
                        .expect("CanvasRenderingStrategy: could not find layer for rendering");
                    let data = datas
                        .get(id)
                        .expect("CanvasRenderingStrategy: could not find data for rendering");
                    Self::render_layer(layer, &data.canvas, framework, renderer);
                }
                LayerItem::Group(items, ..) => {
                    Self::render_impl(width, height, items, layers, datas, framework, renderer);
                }
            }
        }
    }
}
