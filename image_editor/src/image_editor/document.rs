use super::layers::{Layer, LayerIndex, RootLayer};
use crate::{
    layers::{
        BitmapLayer, BitmapLayerConfiguration, LayerCreationInfo, LayerDrawContext, LayerTree,
        ShaderLayerSettings,
    },
    LayerConstructionInfo,
};
use cgmath::{num_traits::Num, point2, vec2, Vector2};
use framework::{Framework, TypedBuffer, TypedBufferConfiguration};
use image::DynamicImage;
use std::{collections::HashMap, iter::FromIterator, num::NonZeroU32};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, CommandEncoder, CommandEncoderDescriptor,
    RenderPassColorAttachment, RenderPassDescriptor,
};

pub struct Document<'framework> {
    framework: &'framework Framework,
    document_size: Vector2<u32>,
    canvas_size: Vector2<u32>,
    layers: HashMap<LayerIndex, Layer<'framework>>,
    tree_root: RootLayer,
    final_layer: Layer<'framework>,

    current_layer_index: LayerIndex,
    settings_bind_group: BindGroup,
    image_data: DynamicImage,
    needs_update: bool,
    needs_cpu_update: bool,
}

pub struct DocumentCreationInfo {
    pub width: u32,
    pub height: u32,
    pub first_layer_color: [f32; 4],
}

pub fn ceil_to<N: Num + Copy + PartialOrd + From<u32>>(n: N, align_to: N) -> N
where
    u32: From<N>,
{
    let d = {
        let m = n % align_to;
        if m > N::from(0) {
            (u32::from(n) / u32::from(align_to)) + 1
        } else {
            return n;
        }
    };

    align_to * N::from(d)
}

impl<'l> Document<'l> {
    pub fn new(config: DocumentCreationInfo, framework: &'l Framework) -> Self {
        let row_aligned_width = ceil_to(config.width, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let final_layer = BitmapLayer::new(
            &framework,
            BitmapLayerConfiguration {
                label: "Final Rendering Layer".to_owned(),
                width: row_aligned_width,
                height: config.height,
                initial_background_color: [0.5, 0.5, 0.5, 1.0],
            },
        );
        let first_layer = BitmapLayer::new(
            &framework,
            BitmapLayerConfiguration {
                label: "Layer 0".to_owned(),
                width: row_aligned_width,
                height: config.height,
                initial_background_color: [1.0, 1.0, 1.0, 1.0],
            },
        );
        let first_layer = Layer::new_bitmap(
            first_layer,
            LayerCreationInfo {
                name: "Layer 0".to_owned(),
                position: point2(0.0, 0.0),
                scale: vec2(1.0, 1.0),
                rotation_radians: 0.0,
            },
            &framework,
        );

        let first_layer_index = LayerIndex(0);
        let mut final_layer = Layer::new_bitmap(
            final_layer,
            LayerCreationInfo {
                name: "Test Layer".to_owned(),
                position: point2(0.0, 0.0),
                scale: vec2(1.0, 1.0),
                rotation_radians: 0.0,
            },
            &framework,
        );
        final_layer.update();
        let settings_group_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Layer Draw Settings bind layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let settings_buffer = framework.allocate_typed_buffer(TypedBufferConfiguration {
            initial_setup: framework::typed_buffer::BufferInitialSetup::Data(&vec![
                ShaderLayerSettings { opacity: 1.0 },
            ]),
            buffer_type: framework::BufferType::Uniform,
            allow_write: true,
            allow_read: false,
        });
        let settings_bind_group = framework.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Layer Settings Bind Group"),
            layout: &settings_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(settings_buffer.binding_resource()),
            }],
        });
        Self {
            framework,
            document_size: vec2(config.width, config.height),
            canvas_size: vec2(row_aligned_width, config.height),
            current_layer_index: first_layer_index,
            final_layer,
            layers: HashMap::from_iter(std::iter::once((first_layer_index, first_layer))),
            tree_root: RootLayer(vec![LayerTree::SingleLayer(first_layer_index)]),
            settings_bind_group,
            image_data: DynamicImage::new_rgba32f(config.width, config.height),
            needs_update: true,
            needs_cpu_update: true,
        }
    }

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

    pub(crate) fn add_layer(
        &mut self,
        framework: &'l Framework,
        layer_name: String,
        layer_index: LayerIndex,
        config: LayerConstructionInfo,
    ) {
        let new_layer = BitmapLayer::new(
            framework,
            BitmapLayerConfiguration {
                label: layer_name.clone(),
                width: self.canvas_size().x,
                height: self.canvas_size().y,
                initial_background_color: config.initial_color,
            },
        );
        let new_layer = Layer::new_bitmap(
            new_layer,
            LayerCreationInfo {
                name: config.name,
                position: point2(0.0, 0.0),
                scale: vec2(1.0, 1.0),
                rotation_radians: 0.0,
            },
            framework,
        );
        self.layers.insert(layer_index.clone(), new_layer);
        self.tree_root.0.push(LayerTree::SingleLayer(layer_index));
    }

    pub(crate) fn update_layers(&mut self) {
        if !self.needs_update {
            return;
        }
        for (_, layer) in self.layers.iter_mut() {
            layer.update();
        }
    }

    pub(crate) fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        layer_draw_pass: &crate::layers::LayerDrawPass,
    ) {
        if !self.needs_update {
            return;
        }
        self.needs_update = false;
        {
            {
                let render_pass_description = RenderPassDescriptor {
                    label: Some("ImageEditor Clear Final Image"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: self.final_layer_texture_view(),
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                };
                let _ = encoder.begin_render_pass(&render_pass_description);
            }
        }
        let render_pass_description = RenderPassDescriptor {
            label: Some("ImageEditor Redraw Image Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: self.final_layer_texture_view(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        };
        for layer_node in self.tree_root.0.iter() {
            match layer_node {
                LayerTree::SingleLayer(index) => {
                    let layer = self.layers.get(&index).expect("Nonexistent layer");
                    let mut render_pass = encoder.begin_render_pass(&render_pass_description);
                    layer_draw_pass.prepare(&mut render_pass);
                    layer.draw(LayerDrawContext {
                        render_pass,
                        draw_pass: &layer_draw_pass,
                    });
                }
                LayerTree::Group(indices) => {
                    for index in indices {
                        let layer = self.layers.get(index).expect("Nonexistent layer");
                        let render_pass = encoder.begin_render_pass(&render_pass_description);
                        layer.draw(LayerDrawContext {
                            render_pass,
                            draw_pass: &layer_draw_pass,
                        });
                    }
                }
            };
        }
    }

    fn update_cpu_image(&mut self) -> &DynamicImage {
        self.needs_cpu_update = false;
        let final_image_size = self.canvas_size();
        let bytes_per_row = final_image_size.x as u32 * 4;
        let final_image_bytes = bytes_per_row * final_image_size.y as u32;
        let mut encoder = self
            .framework
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Fetch final texture"),
            });
        let final_buffer = TypedBuffer::new(
            &self.framework,
            TypedBufferConfiguration {
                initial_setup: framework::typed_buffer::BufferInitialSetup::<u8>::Size(
                    final_image_bytes as u64,
                ),
                buffer_type: framework::BufferType::Oneshot,
                allow_write: true,
                allow_read: true,
            },
        );
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: self.final_texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: final_buffer.inner_buffer(),
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(bytes_per_row),
                    rows_per_image: NonZeroU32::new(final_image_size.y as u32),
                },
            },
            wgpu::Extent3d {
                width: final_image_size.x as u32,
                height: final_image_size.y as u32,
                depth_or_array_layers: 1,
            },
        );
        self.framework
            .queue
            .submit(std::iter::once(encoder.finish()));
        self.framework.device.poll(wgpu::Maintain::Wait);
        let bytes = final_buffer.read_all_sync();
        let buffer = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
            final_image_size.x as u32,
            final_image_size.y as u32,
            bytes,
        )
        .expect("Invalid data from GPU!");
        let document_size = self.document_size();
        let canvas_size = self.canvas_size();
        let diff_x = canvas_size.x - document_size.x;
        let offset_x = diff_x / 2;
        // TODO: We shouldn't flip the image, but rather the images should be rendered correctly
        let image = image::DynamicImage::ImageRgba8(buffer).flipv().crop(
            offset_x,
            0,
            document_size.x,
            document_size.y,
        );
        self.image_data = image;
        &self.image_data
    }

    pub fn mark_dirty(&mut self) {
        self.needs_update = true;
        self.needs_cpu_update = true;
    }

    pub fn final_layer(&self) -> &Layer {
        &self.final_layer
    }

    pub fn settings_bind_group(&self) -> &BindGroup {
        &self.settings_bind_group
    }

    pub fn document_size(&self) -> Vector2<u32> {
        self.document_size
    }

    pub fn current_layer_index(&self) -> LayerIndex {
        self.current_layer_index
    }

    pub(crate) fn canvas_size(&self) -> Vector2<u32> {
        self.canvas_size
    }

    pub fn image_bytes(&mut self) -> &DynamicImage {
        if self.needs_cpu_update {
            self.update_cpu_image();
        }
        &self.image_data
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
}
