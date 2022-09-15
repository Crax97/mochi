use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    num::NonZeroU32,
    rc::Rc,
};

use cgmath::{num_traits::Num, point2, vec2, Point2, Vector2};
use framework::{render_pass::RenderPass, Framework, TypedBuffer, TypedBufferConfiguration};
use scene::Camera2d;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, CommandBuffer, CommandEncoder,
    CommandEncoderDescriptor, RenderPassColorAttachment, RenderPassDescriptor,
};

use framework::{asset_library::AssetsLibrary, pipeline_names};

use crate::layers::{LayerDrawPass, ShaderLayerSettings};

use super::{
    document::Document,
    layers::{
        BitmapLayer, BitmapLayerConfiguration, Layer, LayerCreationInfo, LayerDrawContext,
        LayerIndex, LayerTree, RootLayer,
    },
};

#[derive(Default)]
pub struct LayerConstructionInfo {
    pub initial_color: [f32; 4],
    pub name: String,
}

pub struct ImageBytes {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub bytes: Vec<u8>,
}

pub struct ImageEditor<'framework> {
    framework: &'framework Framework,
    assets: Rc<RefCell<AssetsLibrary>>,
    pan_camera: Camera2d<'framework>,

    document: Document<'framework>,
    layers_created: u16,
    layer_draw_pass: LayerDrawPass,
    canvas: BitmapLayer<'framework>,
    settings_bind_group: BindGroup,
    camaera_bind_group: BindGroup,
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

impl<'framework> ImageEditor<'framework> {
    pub fn new(
        framework: &'framework Framework,
        assets: Rc<RefCell<AssetsLibrary>>,
        initial_window_bounds: &[f32; 2],
    ) -> Self {
        let test_width = 1800;
        let test_height = 1024;
        let ceiled_width = ceil_to(test_width, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let final_layer = BitmapLayer::new(
            &framework,
            BitmapLayerConfiguration {
                label: "Final Rendering Layer".to_owned(),
                width: ceiled_width,
                height: test_height,
                initial_background_color: [0.5, 0.5, 0.5, 1.0],
            },
        );
        let canvas = BitmapLayer::new(
            &framework,
            BitmapLayerConfiguration {
                label: "ImageEditor Canvas".to_owned(),
                width: ceiled_width,
                height: test_height,
                initial_background_color: [0.5, 0.5, 0.5, 1.0],
            },
        );
        let test_layer = BitmapLayer::new(
            &framework,
            BitmapLayerConfiguration {
                label: "Layer 0".to_owned(),
                width: ceiled_width,
                height: test_height,
                initial_background_color: [1.0, 1.0, 1.0, 1.0],
            },
        );

        let left_right_top_bottom = [
            -initial_window_bounds[0] * 0.5,
            initial_window_bounds[0] * 0.5,
            initial_window_bounds[1] * 0.5,
            -initial_window_bounds[1] * 0.5,
        ];
        let mut pan_camera = Camera2d::new(-0.1, 1000.0, left_right_top_bottom, &framework);
        let initial_camera_scale = if initial_window_bounds[0] > initial_window_bounds[1] {
            test_layer.size().x / initial_window_bounds[0]
        } else {
            test_layer.size().y / initial_window_bounds[1]
        } * 1.5;
        println!("Initial scale: {initial_camera_scale}");
        pan_camera.set_scale(initial_camera_scale);

        let test_layer_index = LayerIndex(0);
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

        let settings_buffer = framework.allocate_typed_buffer(TypedBufferConfiguration {
            initial_setup: framework::typed_buffer::BufferInitialSetup::Data(&vec![
                ShaderLayerSettings { opacity: 1.0 },
            ]),
            buffer_type: framework::BufferType::Uniform,
            allow_write: true,
            allow_read: false,
        });

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
        let settings_bind_group = framework.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Layer Settings Bind Group"),
            layout: &settings_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(settings_buffer.binding_resource()),
            }],
        });

        final_layer.update();
        let test_document = Document {
            width: ceiled_width,
            height: test_height,
            layers: HashMap::from_iter(std::iter::once((
                test_layer_index,
                Layer::new_bitmap(
                    test_layer,
                    LayerCreationInfo {
                        name: "Background Layer".to_owned(),
                        position: point2(0.0, 0.0),
                        scale: vec2(1.0, 1.0),
                        rotation_radians: 0.0,
                    },
                    &framework,
                ),
            ))),
            tree_root: RootLayer(vec![LayerTree::SingleLayer(test_layer_index)]),
            final_layer,
            current_layer_index: test_layer_index,
        };
        let layer_draw_pass = LayerDrawPass::new(framework, assets.clone());

        let camera_bind_group_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Layer render pass bind layout"),
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
        let camaera_bind_group = framework
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Layer Camera render pass"),
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        pan_camera
                            .buffer()
                            .inner_buffer()
                            .as_entire_buffer_binding(),
                    ),
                }],
            });
        ImageEditor {
            framework,
            assets,
            canvas,
            pan_camera,
            document: test_document,
            layers_created: 0,
            layer_draw_pass,
            camaera_bind_group,
            settings_bind_group,
        }
    }

    pub fn framework(&'framework self) -> &'framework Framework {
        self.framework
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn mutate_document(&mut self) -> &mut Document<'framework> {
        &mut self.document
    }

    pub fn add_layer_to_document(&mut self, config: LayerConstructionInfo) {
        let layer_name = format!("Layer {}", self.layers_created);
        self.layers_created += 1;
        let layer_index = LayerIndex(self.layers_created);
        let new_layer = BitmapLayer::new(
            &self.framework,
            BitmapLayerConfiguration {
                label: layer_name.clone(),
                width: self.document.width,
                height: self.document.height,
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
            self.framework,
        );
        self.document.layers.insert(layer_index.clone(), new_layer);
        self.document
            .tree_root
            .0
            .push(LayerTree::SingleLayer(layer_index));
    }

    pub fn select_new_layer(&mut self, layer_idx: LayerIndex) {
        self.document.select_layer(layer_idx);
    }

    pub fn delete_layer(&mut self, layer_idx: LayerIndex) {
        self.document.delete_layer(layer_idx);
    }

    pub fn on_resize(&mut self, new_bounds: [f32; 4]) {
        self.pan_camera.set_new_bounds(new_bounds);
    }

    pub fn update_layers(&mut self) {
        for (_, layer) in self.document.layers.iter_mut() {
            layer.update();
        }
    }

    pub fn redraw_full_image(&mut self) -> CommandBuffer {
        let command_encoder_description = CommandEncoderDescriptor {
            label: Some("Image render encoder"),
        };
        let mut command_encoder = self
            .framework
            .device
            .create_command_encoder(&command_encoder_description);

        let old_scale = self.pan_camera.current_scale();
        let old_pos = self.pan_camera.position();

        self.pan_camera.set_scale(1.0);
        self.pan_camera.set_position(point2(0.0, 0.0));
        self.render_document(&mut command_encoder);

        self.pan_camera.set_scale(old_scale);
        self.pan_camera.set_position(old_pos);
        self.render_canvas(&mut command_encoder);

        command_encoder.finish()
    }

    fn render_document(&mut self, encoder: &mut CommandEncoder) {
        {
            let render_pass_description = RenderPassDescriptor {
                label: Some("ImageEditor Clear Final Image"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: self.document.final_layer_texture_view(),
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
        let render_pass_description = RenderPassDescriptor {
            label: Some("ImageEditor Redraw Image Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: self.document.final_layer_texture_view(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        };
        {
            for layer_node in self.document.tree_root.0.iter() {
                match layer_node {
                    LayerTree::SingleLayer(index) => {
                        let layer = self.document.layers.get(&index).expect("Nonexistent layer");
                        let mut render_pass = encoder.begin_render_pass(&render_pass_description);
                        self.layer_draw_pass.prepare(&mut render_pass);
                        layer.draw(LayerDrawContext {
                            render_pass,
                            assets: self.assets.borrow(),
                            draw_pass: &self.layer_draw_pass,
                        });
                    }
                    LayerTree::Group(indices) => {
                        for index in indices {
                            let layer = self.document.layers.get(index).expect("Nonexistent layer");
                            let render_pass = encoder.begin_render_pass(&render_pass_description);
                            layer.draw(LayerDrawContext {
                                render_pass,
                                assets: self.assets.borrow(),
                                draw_pass: &self.layer_draw_pass,
                            });
                        }
                    }
                };
            }
        }
    }

    fn render_canvas(&mut self, command_encoder: &mut CommandEncoder) {
        let render_pass_description = RenderPassDescriptor {
            label: Some("ImageEditor Canvas Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: self.get_full_image_texture().texture_view(),
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
        {
            let mut render_pass = command_encoder.begin_render_pass(&render_pass_description);
            self.layer_draw_pass.prepare(&mut render_pass);
            self.layer_draw_pass.execute_with_renderpass(
                render_pass,
                &[
                    (1, &self.document.final_layer.instance_buffer),
                    (0, self.document.final_layer.bind_group()),
                    (1, &self.camaera_bind_group),
                    (2, &self.settings_bind_group),
                ],
            );
        }
    }

    pub fn get_full_image_texture(&self) -> &BitmapLayer {
        &self.canvas
    }

    pub fn get_full_image_bytes(&self) -> image::DynamicImage {
        let final_image_size = self.document.final_layer.size();
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
                texture: self.document.final_texture(),
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
        let image = image::DynamicImage::ImageRgba8(buffer).flipv();
        image
    }

    pub(crate) fn pan_camera(&mut self, delta: cgmath::Vector2<f32>) {
        let delta = self.pan_camera.current_scale() * delta;
        let half_outer_size = self.document.outer_size();

        let mut new_position = self.pan_camera.position() + delta;
        new_position.x = new_position.x.clamp(-half_outer_size.x, half_outer_size.x);
        new_position.y = new_position.y.clamp(-half_outer_size.y, half_outer_size.y);
        self.pan_camera.set_position(new_position);
    }

    pub fn scale_view(&mut self, delta: f32) {
        const SCALE_SPEED: f32 = 100.0; // TODO: Make this customizable
        self.pan_camera.scale(delta * SCALE_SPEED);
    }

    pub(crate) fn selected_layer(&self) -> &Layer {
        self.document.current_layer()
    }

    pub(crate) fn assets(&self) -> Ref<AssetsLibrary> {
        self.assets.borrow()
    }

    pub fn camera(&self) -> &Camera2d {
        &self.pan_camera
    }
    pub fn camera_mut(&mut self) -> &mut Camera2d<'framework> {
        &mut self.pan_camera
    }
}
