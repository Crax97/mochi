use crate::image_editor::ab_render_target::ABRenderTarget;
use crate::layers::{CanvasRenderingStrategy, Layer, LayerId, LayerItem, LayerRenderingStrategy};
use crate::{
    global_selection_data,
    layers::{LayerCreationInfo, LayerTree, LayerType},
    selection::{Selection, SelectionAddition, SelectionShape},
    LayerConstructionInfo,
};
use cgmath::{point2, vec2, SquareMatrix, Vector2};
use framework::{
    framework::DepthStencilTextureId,
    renderer::draw_command::{DrawCommand, DrawMode, OptionalDrawData, PrimitiveType},
    Framework,
};
use framework::{
    framework::TextureId,
    renderer::{
        draw_command::BindableResource,
        renderer::{DepthStencilUsage, Renderer},
    },
    scene::Camera2d,
    Box2d, DepthStencilTexture2D, RgbaTexture2D, Texture, TextureConfiguration, TextureUsage,
};
use framework::{math, RgbaU8};
use image::{DynamicImage, ImageBuffer};

pub struct SelectionLayer {
    pub layer: Layer,
    pub original_layer: LayerId,
}

pub struct Document {
    document_size: Vector2<u32>,
    tree: LayerTree<Layer>,
    rendering_strategy: CanvasRenderingStrategy,
    selection_layer: Option<SelectionLayer>,

    #[allow(dead_code)]
    buffer_texture: TextureId,
    selection: Selection,
    partial_selection: Selection,
    wants_selection_update: bool,
    stencil_texture: DepthStencilTextureId,
    render_result: TextureId,
}

pub struct DocumentCreationInfo {
    pub width: u32,
    pub height: u32,
    pub first_layer_color: [f32; 4],
}

impl Document {
    pub fn new(config: DocumentCreationInfo, framework: &mut Framework) -> Self {
        let stencil_texture = framework.allocate_depth_stencil_texture(
            DepthStencilTexture2D::empty((config.width, config.height)),
            TextureConfiguration {
                label: Some("Selection stencil texture"),
                usage: TextureUsage::RWRT,
                mip_count: None,
            },
        );
        let mut document = Self {
            document_size: vec2(config.width, config.height),
            selection_layer: None,
            buffer_texture: framework.allocate_texture2d(
                RgbaTexture2D::empty((config.width, config.height)),
                TextureConfiguration {
                    label: Some("document buffer texture"),
                    usage: TextureUsage::RWRT,
                    mip_count: None,
                },
            ),
            selection: Selection::default(),
            partial_selection: Selection::default(),
            wants_selection_update: false,
            stencil_texture,
            tree: LayerTree::new(),
            rendering_strategy: CanvasRenderingStrategy::new(framework, &config),
            render_result: framework.allocate_texture2d(
                RgbaTexture2D::empty((1, 1)),
                TextureConfiguration {
                    label: None,
                    usage: TextureUsage::RWRT,
                    mip_count: None,
                },
            ),
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

    pub fn current_layer(&self) -> &Layer {
        self.tree.current_layer().unwrap()
    }

    pub fn current_layer_index(&self) -> Option<&LayerId> {
        self.tree.current_layer_id()
    }

    pub fn select_layer(&mut self, new_current_layer: LayerId) {
        self.tree.select_layer(new_current_layer)
    }

    pub fn get_layer(&self, layer_index: &LayerId) -> &Layer {
        self.tree.get_layer(layer_index)
    }

    pub fn mutate_layer<F: FnOnce(&mut Layer)>(&mut self, layer_index: &LayerId, mutate_fn: F) {
        let layer = self.tree.get_layer_mut(layer_index);
        mutate_fn(layer);
    }

    pub fn mutate_selection<F: FnOnce(&mut Selection)>(&mut self, callback: F) {
        callback(&mut self.selection);
        self.wants_selection_update = true;
    }
    pub fn mutate_partial_selection<F: FnOnce(&mut Selection)>(&mut self, callback: F) {
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
            renderer.begin(
                &Camera2d::wh(self.document_size.x, self.document_size.y),
                None,
                framework,
            );
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
                &self.buffer_texture,
                Some((&self.stencil_texture, DepthStencilUsage::Stencil)),
                framework,
            );
        }
    }

    fn clear_stencil_buffer(&self, renderer: &mut Renderer, framework: &mut Framework) {
        renderer.begin(
            &Camera2d::default(),
            Some(wgpu::Color::TRANSPARENT),
            framework,
        );
        renderer.set_draw_debug_name("Selection tool: clear stencil buffer");
        renderer.set_stencil_clear(Some(0));
        renderer.end(
            &self.buffer_texture,
            Some((&self.stencil_texture, DepthStencilUsage::Stencil)),
            framework,
        );
    }

    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    pub fn draw_selection(&self, renderer: &mut Renderer) {
        let extents = self.document_size.cast::<f32>().unwrap() * 0.5;
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
        layer_below_idx: &LayerId,
        layer_top_idx: &LayerId,
        renderer: &mut Renderer,
        framework: &mut Framework,
    ) {
        let layer_below = self.get_layer(layer_below_idx);
        let layer_top = self.get_layer(layer_top_idx);

        join_bitmaps(&layer_below, &layer_top, renderer, framework);
    }

    pub fn join_with_layer_below(
        &mut self,
        top: &LayerId,
        renderer: &mut Renderer,
        framework: &mut Framework,
    ) {
        let layer = self.tree.find_below(top);
        if let Some(below) = layer {
            self.join_layers(&below, top, renderer, framework)
        }
    }

    pub fn extract_selection(&mut self, renderer: &mut Renderer, framework: &mut Framework) {
        let current_layer = self.current_layer();
        let dims = current_layer.size();
        let dims = (dims.x, dims.y);

        let tex = match current_layer.layer_type {
            LayerType::Image { ref texture, .. } => texture.clone(),
            LayerType::Group => unreachable!(),
        };
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
            &Self::make_camera_for_layer(&current_layer),
            Some(wgpu::Color::TRANSPARENT),
            framework,
        );
        renderer.set_draw_debug_name("Selection tool: draw layer with stencil buffer");
        renderer.set_stencil_clear(None);
        renderer.set_stencil_reference(255);
        renderer.draw(DrawCommand {
            primitives: PrimitiveType::Texture2D {
                texture_id: tex.clone(),
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
            &Self::make_camera_for_layer(&current_layer),
            Some(wgpu::Color::TRANSPARENT),
            framework,
        );
        renderer.set_draw_debug_name("Selection tool: draw layer with inverted stencil buffer");
        renderer.set_stencil_clear(None);
        renderer.set_stencil_reference(255);
        renderer.draw(DrawCommand {
            primitives: PrimitiveType::Texture2D {
                texture_id: tex,
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
        let current_layer = self.current_layer_index().copied().unwrap();
        //5. Now add the new layer
        let mut new_layer = SelectionLayer {
            layer: Layer::new_image(
                RgbaTexture2D::from_repeated_texel(
                    RgbaU8([0; 4]),
                    (self.document_size.x, self.document_size.y),
                )
                .unwrap(),
                LayerCreationInfo {
                    name: "Selection layer".to_owned(),
                    position: point2(0.0, 0.0),
                    scale: vec2(1.0, 1.0),
                    rotation_radians: 0.0,
                },
                framework,
            ),
            original_layer: current_layer.clone(),
        };
        new_layer
            .layer
            .set_settings(self.current_layer().settings().clone());
        self.selection_layer = Some(new_layer);
        self.mutate_layer(&current_layer, |layer| {
            layer.replace_texture(old_texture_copy.clone())
        });

        self.selection.clear();
        self.update_selection_buffer(renderer, framework);
    }

    pub fn selection_layer_mut(&mut self) -> Option<&mut SelectionLayer> {
        self.selection_layer.as_mut()
    }
    pub fn selection_layer(&self) -> Option<&SelectionLayer> {
        self.selection_layer.as_ref()
    }

    pub fn apply_selection(&mut self, renderer: &mut Renderer, framework: &mut Framework) {
        if !self.selection_layer.is_some() {
            return;
        }
        let selection = self.selection_layer.take().unwrap();
        let layer_below = self.get_layer(&selection.original_layer);
        join_bitmaps(layer_below, &selection.layer, renderer, framework);
    }

    pub fn delete_layer(&mut self, layer_idx: LayerId) {
        let layer = self.tree.remove_layer(layer_idx);
        self.rendering_strategy.on_layer_removed(&layer);
    }

    pub(crate) fn add_layer(
        &mut self,
        config: LayerConstructionInfo,
        framework: &mut Framework,
    ) -> LayerId {
        let new_layer = Layer::new_image(
            RgbaTexture2D::from_repeated_texel(
                RgbaU8(config.initial_color),
                (self.document_size.x, self.document_size.y),
            )
            .unwrap(),
            LayerCreationInfo {
                name: config.name.clone(),
                position: point2(0.0, 0.0),
                scale: vec2(1.0, 1.0),
                rotation_radians: 0.0,
            },
            framework,
        );
        let id = new_layer.id().clone();
        self.rendering_strategy.on_new_layer(&new_layer, framework);
        self.tree.add_layer(new_layer);
        id
    }

    pub(crate) fn update_layers(&mut self, renderer: &mut Renderer, framework: &mut Framework) {
        if self.wants_selection_update {
            self.wants_selection_update = false;
            self.update_selection_buffer(renderer, framework);
        }
        self.rendering_strategy.update(&self.tree.layers, framework);
    }

    pub(crate) fn render(&mut self, renderer: &mut Renderer, framework: &mut Framework) {
        self.rendering_strategy.update_canvases(
            &self.tree.items,
            &self.tree.layers,
            framework,
            renderer,
        );
        self.render_result = self.composite_final_image(
            self.document_size.x,
            self.document_size.y,
            renderer,
            framework,
        );
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

    pub fn document_size(&self) -> Vector2<u32> {
        self.document_size
    }

    pub fn final_image_bytes(&self, framework: &Framework) -> DynamicImage {
        let texture = framework.texture2d_read_data(&self.render_result);
        let width = texture.width();
        let height = texture.height();
        let bytes = texture
            .data()
            .expect("A texture just read from the GPU doesn'thave any bytes, wtf?");
        let bytes = bytemuck::cast_slice(bytes).to_owned();
        let raw_image = ImageBuffer::from_raw(width, height, bytes).unwrap();
        DynamicImage::ImageRgba8(raw_image)
    }

    pub fn for_each_layer<F: FnMut(&Layer, &LayerId)>(&self, mut f: F) {
        self.tree.for_each_layer(|l| f(l, &l.id().clone()));
    }

    pub fn render_camera(&self) -> Camera2d {
        let half_w = self.document_size.x as f32 * 0.5;
        let half_h = self.document_size.y as f32 * 0.5;
        Camera2d::new(-0.01, 1000.0, [-half_w, half_w, half_h, -half_h])
    }

    pub fn make_camera_for_layer(layer: &Layer) -> Camera2d {
        let extents = layer.bounds().extents;
        Camera2d::new(
            -0.01,
            1000.0,
            [-extents.x, extents.x, extents.y, -extents.y],
        )
    }

    pub fn render_result(&self) -> &TextureId {
        &self.render_result
    }

    pub fn composite_final_image(
        &mut self,
        width: u32,
        height: u32,
        renderer: &mut Renderer,
        framework: &mut Framework,
    ) -> TextureId {
        Self::composite_final_image_impl(
            &self.tree.items,
            &self.rendering_strategy,
            width,
            height,
            renderer,
            framework,
        )
    }

    fn composite_final_image_impl<T: LayerRenderingStrategy<Layer>>(
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

fn join_bitmaps(
    layer_below: &Layer,
    layer_top: &Layer,
    renderer: &mut Renderer,
    framework: &mut Framework,
) {
    match (&layer_top.layer_type, &layer_below.layer_type) {
        (
            LayerType::Image { texture: top, .. },
            LayerType::Image {
                texture: bottom, ..
            },
        ) => {
            let below_inverse_transform = layer_below
                .transform()
                .matrix()
                .invert()
                .expect("Failed to invert matrix in join layers!");
            let adjusted_top_transform = layer_top.transform().matrix() * below_inverse_transform;
            let mut transform = math::helpers::decompose_no_shear_2d(adjusted_top_transform);
            transform.scale(layer_top.bounds().extents);
            renderer.begin(
                &Document::make_camera_for_layer(layer_below),
                None,
                framework,
            );
            renderer.draw(DrawCommand {
                primitives: PrimitiveType::Texture2D {
                    texture_id: top.clone(),
                    instances: vec![transform],
                    flip_uv_y: false,
                    multiply_color: wgpu::Color::WHITE,
                },
                draw_mode: DrawMode::Single,
                additional_data: OptionalDrawData::default(),
            });

            renderer.end(&bottom, None, framework);
        }
        _ => log::warn!(
            "Cannot join layer of type {:?} with layer of type {:?}",
            layer_below.layer_type,
            layer_top.layer_type
        ),
    }
}
