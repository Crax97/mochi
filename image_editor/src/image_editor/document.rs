use super::layers::{Layer, LayerIndex, RootLayer};
use crate::{
    blend_settings::{BlendSettings, BlendSettingsUniform},
    global_selection_data,
    layers::{BitmapLayer, BitmapLayerConfiguration, LayerCreationInfo, LayerTree},
    selection::{Selection, SelectionAddition, SelectionShape},
    LayerConstructionInfo,
};
use cgmath::{
    point2, point3, vec2, vec3, Decomposed, InnerSpace, Matrix, Matrix3, Matrix4, SquareMatrix,
    Transform, Vector2,
};
use framework::{
    framework::TextureId,
    math,
    renderer::{
        draw_command::BindableResource,
        renderer::{DepthStencilUsage, Renderer},
    },
    scene::Camera2d,
    Box2d, BufferConfiguration, DepthStencilTexture2D, RgbaTexture2D, Texture,
    TextureConfiguration, TextureUsage,
};
use framework::{
    framework::{BufferId, DepthStencilTextureId},
    renderer::draw_command::{DrawCommand, DrawMode, OptionalDrawData, PrimitiveType},
    Framework,
};
use image::{DynamicImage, ImageBuffer};

use framework::framework::ShaderId;
use std::collections::HashMap;
use std::slice::Iter;
use uuid::Uuid;

enum BufferingStep {
    First,
    Second,
}

struct LayerDrawInfo {
    bitmap_canvas: BitmapLayer,
    layer_settings_buffer: BufferId,
}

pub struct Document {
    layers_created: u16,

    document_size: Vector2<u32>,
    layers: HashMap<LayerIndex, Layer>,
    layer_canvases: HashMap<Uuid, LayerDrawInfo>,
    tree_root: RootLayer,
    final_layer_1: BitmapLayer,
    final_layer_2: BitmapLayer,
    #[allow(dead_code)]
    buffer_layer: BitmapLayer, // Imma keep it here just in case, too many times i removed it just to need it later again
    buffering_step: BufferingStep,

    current_layer_index: LayerIndex,
    selection: Selection,
    partial_selection: Selection,
    wants_selection_update: bool,
    stencil_texture: DepthStencilTextureId,
}

pub struct DocumentCreationInfo {
    pub width: u32,
    pub height: u32,
    pub first_layer_color: [f32; 4],
}

impl Document {
    pub fn new(config: DocumentCreationInfo, framework: &mut Framework) -> Self {
        let final_layer_1 = BitmapLayer::new(
            "Double Buffering Layer 1",
            [127, 127, 127, 127],
            BitmapLayerConfiguration {
                width: config.width,
                height: config.height,
            },
            framework,
        );
        let final_layer_2 = BitmapLayer::new(
            "Double Buffering Layer 2",
            [127, 127, 127, 127],
            BitmapLayerConfiguration {
                width: config.width,
                height: config.height,
            },
            framework,
        );
        let buffer_layer = BitmapLayer::new(
            "Draw Buffer Layer",
            [0, 0, 0, 0],
            BitmapLayerConfiguration {
                width: config.width,
                height: config.height,
            },
            framework,
        );

        let first_layer_index = LayerIndex(1);
        let stencil_texture = framework.allocate_depth_stencil_texture(
            DepthStencilTexture2D::empty((config.width, config.height)),
            TextureConfiguration {
                label: Some("Selection stencil texture"),
                usage: TextureUsage::RWRT,
                mip_count: None,
            },
        );
        let mut document = Self {
            layers_created: 0,
            document_size: vec2(config.width, config.height),
            current_layer_index: first_layer_index,

            final_layer_1,
            final_layer_2,
            buffer_layer,

            layers: HashMap::new(),
            layer_canvases: HashMap::new(),
            tree_root: RootLayer(vec![]),
            buffering_step: BufferingStep::First,
            selection: Selection::default(),
            partial_selection: Selection::default(),
            wants_selection_update: false,
            stencil_texture,
        };

        document.add_layer(
            LayerConstructionInfo {
                initial_color: [255; 4],
                name: "Background Layer".into(),
            },
            framework,
        );
        document.add_layer(
            LayerConstructionInfo {
                initial_color: [0; 4],
                name: "Layer 0".into(),
            },
            framework,
        );

        document
    }

    pub fn outer_size(&self) -> Vector2<f32> {
        self.final_layer().size()
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

    pub fn mutate_layer<F: FnMut(&mut Layer)>(
        &mut self,
        layer_index: &LayerIndex,
        mut mutate_fn: F,
    ) {
        let layer = self
            .layers
            .get_mut(&layer_index)
            .expect("Invalid layer index passed to document!");

        mutate_fn(layer);
    }

    pub fn mutate_selection<F: FnMut(&mut Selection)>(&mut self, mut callback: F) {
        callback(&mut self.selection);
        self.wants_selection_update = true;
    }
    pub fn mutate_partial_selection<F: FnMut(&mut Selection)>(&mut self, mut callback: F) {
        callback(&mut self.partial_selection);
        self.wants_selection_update = true;
    }

    fn update_selection_buffer(&self, renderer: &mut Renderer, framework: &mut Framework) {
        self.clear_stencil_buffer(renderer, framework);
        self.draw_shapes_on_stencil_buffer(&self.selection.shapes, renderer, framework);
        self.draw_shapes_on_stencil_buffer(&self.partial_selection.shapes, renderer, framework);
    }

    fn draw_shapes_on_stencil_buffer<'a, T: IntoIterator<Item = &'a SelectionShape>>(
        &self,
        shapes: T,
        renderer: &mut Renderer,
        framework: &mut Framework,
    ) {
        for shape in shapes.into_iter() {
            let additive = shape.mode == SelectionAddition::Add;
            renderer.begin(&self.buffer_layer.camera(), None, framework);
            renderer.set_draw_debug_name(
                format!(
                    "Selection tool: draw shape {:?} [{:?}] on stencil buffer",
                    shape.shape,
                    if additive { "a" } else { "s" }
                )
                .as_str(),
            );
            renderer.set_stencil_clear(None);
            renderer.set_stencil_reference(if additive { 255 } else { 0 });
            match shape.shape {
                crate::selection::Shape::Rectangle(rect) => {
                    renderer.draw(DrawCommand {
                        primitives: PrimitiveType::Rect {
                            rects: vec![rect.clone()],
                            multiply_color: wgpu::Color::GREEN,
                        },
                        draw_mode: DrawMode::Single,
                        additional_data: OptionalDrawData::just_shader(Some(
                            global_selection_data()
                                .draw_on_stencil_buffer_shader_id
                                .clone(),
                        )),
                    });
                }
            }

            renderer.end(
                &self.buffer_layer.texture(),
                Some((&self.stencil_texture, DepthStencilUsage::Stencil)),
                framework,
            );
        }
    }

    fn clear_stencil_buffer(&self, renderer: &mut Renderer, framework: &mut Framework) {
        renderer.begin(
            &self.buffer_layer.camera(),
            Some(wgpu::Color::TRANSPARENT),
            framework,
        );
        renderer.set_draw_debug_name("Selection tool: clear stencil buffer");
        renderer.set_stencil_clear(Some(0));
        renderer.end(
            &self.buffer_layer.texture(),
            Some((&self.stencil_texture, DepthStencilUsage::Stencil)),
            framework,
        );
    }

    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    pub fn draw_selection(&self, renderer: &mut Renderer) {
        let extents = self.final_layer().size() * 0.5;
        renderer.draw(DrawCommand {
            primitives: PrimitiveType::Rect {
                rects: vec![Box2d {
                    center: point2(0.0, 0.0),
                    extents,
                }],
                multiply_color: wgpu::Color::RED,
            },
            draw_mode: DrawMode::Single,
            additional_data: OptionalDrawData {
                additional_vertex_buffers: vec![],
                additional_bindable_resource: vec![BindableResource::StencilTexture(
                    self.stencil_texture.clone(),
                )],
                shader: Some(global_selection_data().dotted_shader.clone()),
            },
        });
    }

    pub fn join_layers(
        &mut self,
        layer_below_idx: &LayerIndex,
        layer_top_idx: &LayerIndex,
        renderer: &mut Renderer,
        framework: &mut Framework,
    ) {
        let layer_below = self.get_layer(layer_below_idx);
        let layer_top = self.get_layer(layer_top_idx);
        let below_inverse_transform = layer_below
            .transform()
            .matrix()
            .invert()
            .expect("Failed to invert matrix in join layers!");
        let adjusted_top_transform = layer_top.transform().matrix() * below_inverse_transform;
        let transform = math::helpers::decompose_no_shear_2d(adjusted_top_transform);

        renderer.begin(&layer_below.bitmap.camera(), None, framework);
        layer_top.bitmap.draw(
            renderer,
            point2(transform.position.x, transform.position.y),
            transform.scale,
            transform.rotation_radians.0,
            layer_top.settings().opacity,
        );
        renderer.end(layer_below.bitmap.texture(), None, framework);
    }

    pub fn join_with_layer_below(
        &mut self,
        top: &LayerIndex,
        renderer: &mut Renderer,
        framework: &mut Framework,
    ) {
        let layers = self.tree_root.0.iter();
        let layer = self.find_layer_below_step_one(top, layers);
        if let Some(below) = layer {
            self.join_layers(&below, top, renderer, framework)
        }
    }

    fn find_layer_below_step_one(
        &self,
        target: &LayerIndex,
        layers: Iter<LayerTree>,
    ) -> Option<LayerIndex> {
        let mut previous = *target;
        for layer_type in layers {
            match layer_type {
                LayerTree::SingleLayer(layer) => {
                    if layer == target {
                        return if &previous == target {
                            // the target layer is the first: there's no layer below
                            None
                        } else {
                            Some(previous)
                        };
                    }
                    previous = *layer;
                }
                LayerTree::Group(layers) => {
                    let mut it = layers.iter();
                    let found = self.find_layer_below_recursive(target, it);
                    if found.is_some() {
                        return found;
                    }
                }
            }
        }
        None
    }

    fn find_layer_below_recursive(
        &self,
        target: &LayerIndex,
        mut layers: Iter<LayerIndex>,
    ) -> Option<LayerIndex> {
        let mut previous = *target;
        for index in layers {
            if target == index {
                // the target layer is the first of a group: there's no layer below
                return None;
            }
            if index == target {
                return Some(previous);
            }
            previous = *index;
        }
        None
    }

    pub fn copy_layer_selection_to_new_layer(
        &mut self,
        renderer: &mut Renderer,
        framework: &mut Framework,
    ) {
        let current_layer = self.current_layer();
        let dims = framework.texture2d_dimensions(current_layer.bitmap.texture());

        let new_texture = framework.allocate_texture2d(
            RgbaTexture2D::empty(dims),
            TextureConfiguration {
                label: Some(&(current_layer.settings().clone().name + " clone texture")),
                usage: TextureUsage::RWRT,
                mip_count: None,
            },
        );
        let old_texture_copy = framework.allocate_texture2d(
            RgbaTexture2D::empty(dims),
            TextureConfiguration {
                label: Some(&(current_layer.settings().clone().name + " texture")),
                usage: TextureUsage::RWRT,
                mip_count: None,
            },
        );

        // 1. Draw layer using the rect stencil buffer, this is the selection. Store it into a new texture
        renderer.begin(
            &current_layer.bitmap.camera(),
            Some(wgpu::Color::TRANSPARENT),
            framework,
        );
        renderer.set_draw_debug_name("Selection tool: draw layer with stencil buffer");
        renderer.set_stencil_clear(None);
        renderer.set_stencil_reference(255);
        renderer.draw(DrawCommand {
            primitives: PrimitiveType::Texture2D {
                texture_id: current_layer.bitmap.texture().clone(),
                instances: vec![current_layer.pixel_transform()],
                flip_uv_y: true,
                multiply_color: wgpu::Color::WHITE,
            },
            draw_mode: DrawMode::Single,
            additional_data: OptionalDrawData::just_shader(Some(if self.selection.inverted {
                global_selection_data()
                    .draw_masked_inverted_stencil_buffer_shader_id
                    .clone()
            } else {
                global_selection_data()
                    .draw_masked_stencil_buffer_shader_id
                    .clone()
            })),
        });
        renderer.end(
            &new_texture,
            Some((&self.stencil_texture, DepthStencilUsage::Stencil)),
            framework,
        );

        // 2. Draw the layer using the inverted stencil buffer: this is the remaining part of the texture
        renderer.begin(
            &current_layer.bitmap.camera(),
            Some(wgpu::Color::TRANSPARENT),
            framework,
        );
        renderer.set_draw_debug_name("Selection tool: draw layer with inverted stencil buffer");
        renderer.set_stencil_clear(None);
        renderer.set_stencil_reference(255);
        renderer.draw(DrawCommand {
            primitives: PrimitiveType::Texture2D {
                texture_id: current_layer.bitmap.texture().clone(),
                instances: vec![current_layer.pixel_transform()],
                flip_uv_y: true,
                multiply_color: wgpu::Color::WHITE,
            },
            draw_mode: DrawMode::Single,
            additional_data: OptionalDrawData::just_shader(Some(if self.selection.inverted {
                global_selection_data()
                    .draw_masked_stencil_buffer_shader_id
                    .clone()
            } else {
                global_selection_data()
                    .draw_masked_inverted_stencil_buffer_shader_id
                    .clone()
            })),
        });
        renderer.end(
            &old_texture_copy,
            Some((&self.stencil_texture, DepthStencilUsage::Stencil)),
            framework,
        );

        //5. Now add the new layer
        let (width, height) = framework.texture2d_dimensions(&new_texture);
        let new_index = self.add_layer(
            LayerConstructionInfo {
                initial_color: [0; 4],
                name: current_layer.settings().name.clone() + " subregion",
            },
            framework,
        );
        self.mutate_layer(&new_index, |layer| {
            layer.replace_texture(new_texture.clone())
        });
        self.mutate_layer(&self.current_layer_index(), |layer| {
            layer.replace_texture(old_texture_copy.clone())
        });

        self.select_layer(new_index);
    }

    pub fn delete_layer(&mut self, layer_idx: LayerIndex) {
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
        let removed_layer = self.layers.remove(&layer_idx).unwrap();
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
        self.layer_canvases.remove(removed_layer.uuid()).unwrap();
    }

    pub(crate) fn add_layer(
        &mut self,
        config: LayerConstructionInfo,
        framework: &mut Framework,
    ) -> LayerIndex {
        let layer_index = LayerIndex(self.layers_created);
        self.layers_created += 1;
        let new_layer = BitmapLayer::new(
            &config.name,
            config.initial_color,
            BitmapLayerConfiguration {
                width: self.document_size.x,
                height: self.document_size.y,
            },
            framework,
        );
        let new_layer = Layer::new_bitmap(
            new_layer,
            LayerCreationInfo {
                name: config.name.clone(),
                position: point2(0.0, 0.0),
                scale: vec2(1.0, 1.0),
                rotation_radians: 0.0,
            },
        );
        let bitmap_canvas = BitmapLayer::new(
            &config.name,
            [0; 4],
            BitmapLayerConfiguration {
                width: self.document_size.x,
                height: self.document_size.y,
            },
            framework,
        );
        let settings = BlendSettingsUniform::from(BlendSettings {
            blend_mode: new_layer.settings().blend_mode,
        });
        let layer_settings_buffer =
            framework.allocate_typed_buffer(BufferConfiguration::<BlendSettingsUniform> {
                initial_setup: framework::buffer::BufferInitialSetup::Data(&vec![settings]),
                buffer_type: framework::BufferType::Uniform,
                gpu_copy_dest: true,
                gpu_copy_source: false,
                cpu_copy_dest: false,
                cpu_copy_source: false,
            });

        let layer_draw_info = LayerDrawInfo {
            bitmap_canvas,
            layer_settings_buffer,
        };

        self.layer_canvases
            .insert(new_layer.uuid().clone(), layer_draw_info);
        self.layers.insert(layer_index.clone(), new_layer);
        self.tree_root
            .0
            .push(LayerTree::SingleLayer(layer_index.clone()));
        layer_index
    }

    pub(crate) fn update_layers(&mut self, renderer: &mut Renderer, framework: &mut Framework) {
        if self.wants_selection_update {
            self.wants_selection_update = false;
            self.update_selection_buffer(renderer, framework);
            self.partial_selection.clear();
        }
        for (_, layer) in self.layers.iter_mut() {
            let layer_info = self.layer_canvases.get(layer.uuid()).unwrap();
            if layer.needs_settings_update() {
                Self::update_layer_settings(layer, &layer_info.layer_settings_buffer, framework);
            }

            if layer.needs_bitmap_update() {
                Self::update_layer_bitmap(renderer, layer, &layer_info.bitmap_canvas, framework);
            }
        }
    }

    pub(crate) fn render(
        &mut self,
        renderer: &mut Renderer,
        shader_to_use: ShaderId,
        framework: &mut Framework,
    ) {
        let draw_sequence = self.generate_draw_sequence();

        self.execute_draw_sequence_double_buffered(
            renderer,
            draw_sequence,
            shader_to_use,
            framework,
        )
    }

    fn execute_draw_sequence_double_buffered(
        &mut self,
        renderer: &mut Renderer,
        draw_sequence: Vec<LayerIndex>,
        shader_to_use: ShaderId,
        framework: &mut Framework,
    ) {
        // Clear first layer
        let final_layer = self.final_layer().texture().clone();
        Self::clear_texture(renderer, &final_layer, wgpu::Color::TRANSPARENT, framework);

        // Actually draw shit
        let mut draw_layer = |index| {
            let final_layer = self.advance_final_layer().texture().clone();
            let previous_layer = self.previous_buffer_layer().texture().clone();

            // 1. Draw current layer onto buffer layer
            let layer = self.get_layer(&index);
            let layer_draw_info = self.layer_canvases.get(layer.uuid()).unwrap();
            // 2. Blend buffer layer with final layer

            layer_draw_info.bitmap_canvas.draw_blended(
                renderer,
                shader_to_use.clone(),
                previous_layer.clone(),
                layer_draw_info.layer_settings_buffer.clone(),
                &final_layer,
                framework,
            );
            Self::clear_texture(
                renderer,
                &previous_layer,
                wgpu::Color::TRANSPARENT,
                framework,
            );
        };
        for layer_index in draw_sequence {
            draw_layer(layer_index);
        }
    }

    pub fn clear_texture(
        renderer: &mut Renderer,
        texture: &TextureId,
        color: wgpu::Color,
        framework: &mut Framework,
    ) {
        renderer.begin(&Camera2d::default(), Some(color), framework);
        renderer.end(texture, None, framework);
    }

    fn generate_draw_sequence(&self) -> Vec<LayerIndex> {
        let mut draw_sequence = Vec::new();
        for layer_node in self.tree_root.0.iter() {
            match layer_node {
                LayerTree::SingleLayer(index) => {
                    let layer = self.get_layer(&index);
                    if !layer.settings().is_enabled {
                        continue;
                    }
                    draw_sequence.push(index.clone());
                }
                LayerTree::Group(indices) => {
                    for index in indices {
                        let layer = self.get_layer(&index);
                        if !layer.settings().is_enabled {
                            continue;
                        }
                        draw_sequence.push(index.clone());
                    }
                }
            };
        }
        draw_sequence
    }

    pub fn final_layer(&self) -> &BitmapLayer {
        match self.buffering_step {
            BufferingStep::First => &self.final_layer_2,
            BufferingStep::Second => &self.final_layer_1,
        }
    }

    pub fn previous_buffer_layer(&self) -> &BitmapLayer {
        match self.buffering_step {
            BufferingStep::First => &self.final_layer_1,
            BufferingStep::Second => &self.final_layer_2,
        }
    }

    fn advance_final_layer(&mut self) -> &BitmapLayer {
        match self.buffering_step {
            BufferingStep::First => {
                self.buffering_step = BufferingStep::Second;
            }
            BufferingStep::Second => {
                self.buffering_step = BufferingStep::First;
            }
        };
        self.final_layer()
    }

    pub fn document_size(&self) -> Vector2<u32> {
        self.document_size
    }

    pub fn current_layer_index(&self) -> LayerIndex {
        self.current_layer_index
    }

    pub fn final_image_bytes(&self, framework: &Framework) -> DynamicImage {
        let texture = framework.texture2d_read_data(self.final_layer().texture());
        let width = texture.width();
        let height = texture.height();
        let bytes = texture
            .data()
            .expect("A texture just read from the GPU doesn'thave any bytes, wtf?");
        let bytes = bytemuck::cast_slice(bytes).to_owned();
        let raw_image = ImageBuffer::from_raw(width, height, bytes).unwrap();
        DynamicImage::ImageRgba8(raw_image)
    }

    pub fn for_each_layer<F: FnMut(&Layer, &LayerIndex)>(&self, mut f: F) {
        for tree in self.tree_root.0.iter() {
            match tree {
                LayerTree::SingleLayer(idx) => {
                    let layer = self.get_layer(idx);
                    f(layer, idx);
                }
                LayerTree::Group(group_members) => {
                    for idx in group_members {
                        let layer = self.get_layer(idx);
                        f(layer, idx);
                    }
                }
            }
        }
    }

    fn update_layer_settings(layer: &mut Layer, target: &BufferId, framework: &mut Framework) {
        framework.buffer_write_sync(
            target,
            vec![BlendSettingsUniform::from(BlendSettings {
                blend_mode: layer.settings().blend_mode,
            })],
        )
    }

    fn update_layer_bitmap(
        renderer: &mut Renderer,
        layer: &mut Layer,
        target: &BitmapLayer,
        framework: &mut Framework,
    ) {
        Self::clear_texture(
            renderer,
            target.texture(),
            wgpu::Color::TRANSPARENT,
            framework,
        );
        layer.lay_on_canvas(renderer, &target, framework);
    }
}
