use std::{
    cell::{Ref, RefCell},
    num::NonZeroU32,
    rc::Rc,
};

use cgmath::{point2, vec2, vec4, ElementWise};
use framework::{
    render_pass::RenderPass, Framework, MeshInstance2D, TypedBuffer, TypedBufferConfiguration,
};
use image::Rgba;
use scene::Camera2d;
use wgpu::{
    BindGroup, CommandBuffer, CommandEncoder, CommandEncoderDescriptor, RenderPassColorAttachment,
    RenderPassDescriptor, TextureView,
};

use framework::asset_library::AssetsLibrary;

use crate::{document::DocumentCreationInfo, RenderToCanvasPass};

use super::{
    document::Document,
    layers::{Layer, LayerIndex},
};

#[derive(Default)]
pub struct LayerConstructionInfo {
    pub initial_color: [u8; 4],
    pub name: String,
}

pub struct ImageEditor<'framework> {
    framework: &'framework Framework,
    assets: Rc<RefCell<AssetsLibrary>>,
    pan_camera: Camera2d<'framework>,

    document: Document,
    layers_created: u16,

    camaera_bind_group: BindGroup,
    canvas_instance_buffer: TypedBuffer<'framework>,
    render_to_canvas_pass: RenderToCanvasPass,
}
impl<'framework> ImageEditor<'framework> {
    pub fn new(
        framework: &'framework Framework,
        assets: Rc<RefCell<AssetsLibrary>>,
        initial_window_bounds: &[f32; 2],
    ) -> Self {
        let test_width = 1800;
        let test_height = 1024;
        let test_document = Document::new(
            DocumentCreationInfo {
                width: test_width,
                height: test_height,
                first_layer_color: [0.0, 0.0, 0.0, 1.0],
            },
            framework,
        );
        let left_right_top_bottom = [
            -initial_window_bounds[0] * 0.5,
            initial_window_bounds[0] * 0.5,
            initial_window_bounds[1] * 0.5,
            -initial_window_bounds[1] * 0.5,
        ];
        let mut pan_camera = Camera2d::new(-0.1, 1000.0, left_right_top_bottom, &framework);
        let initial_camera_scale = if initial_window_bounds[0] > initial_window_bounds[1] {
            test_document.document_size().x as f32 / initial_window_bounds[0]
        } else {
            test_document.document_size().y as f32 / initial_window_bounds[1]
        } * 1.5;
        println!("Initial scale: {initial_camera_scale}");
        pan_camera.set_scale(initial_camera_scale);

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
        let render_to_canvas_pass = RenderToCanvasPass::new(
            framework,
            wgpu::TextureFormat::Bgra8UnormSrgb,
            assets.clone(),
        );
        let canvas_instance_buffer = framework.allocate_typed_buffer(TypedBufferConfiguration {
            initial_setup: framework::typed_buffer::BufferInitialSetup::Data(&vec![
                MeshInstance2D::new(
                    point2(0.0, 0.0),
                    vec2(test_width as f32, test_height as f32),
                    0.0,
                ),
            ]),
            buffer_type: framework::BufferType::Vertex,
            allow_write: true,
            allow_read: false,
        });
        ImageEditor {
            framework,
            assets,
            pan_camera,
            document: test_document,
            layers_created: 0,
            canvas_instance_buffer,
            camaera_bind_group,
            render_to_canvas_pass,
        }
    }

    pub fn framework(&'framework self) -> &'framework Framework {
        self.framework
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn mutate_document(&mut self) -> &mut Document {
        &mut self.document
    }

    pub fn add_layer_to_document(&mut self, config: LayerConstructionInfo) {
        let layer_name = format!("Layer {}", self.layers_created);
        self.layers_created += 1;
        self.document
            .add_layer(layer_name, LayerIndex(self.layers_created), config);
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
        self.mutate_document().update_layers();
    }

    pub fn redraw_full_image(&mut self) {
        self.render_document();
    }

    fn render_document(&mut self) {
        self.document.render();
    }

    pub fn render_canvas(&mut self, target: &TextureView) {
        let mut encoder =
            self.framework
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Image to canvas render pass"),
                });
        let render_pass_description = RenderPassDescriptor {
            label: Some("ImageEditor Canvas Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &target,
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
        self.document().update_gpu_data(&self.framework);
        let render_pass = encoder.begin_render_pass(&render_pass_description);
        self.render_to_canvas_pass.execute_with_renderpass(
            render_pass,
            &[
                (1, &self.canvas_instance_buffer),
                (0, self.document.texture_bind_group()),
                (1, &self.camaera_bind_group),
            ],
        );
        self.framework
            .queue
            .submit(std::iter::once(encoder.finish()));
    }

    pub fn get_full_image_bytes(&mut self) -> &image::ImageBuffer<Rgba<u8>, Vec<u8>> {
        self.mutate_document().image_bytes()
    }

    pub fn pan_camera(&mut self, delta: cgmath::Vector2<f32>) {
        let half_outer_size = self
            .document
            .document_size()
            .cast::<f32>()
            .expect("Somehow this cast failed")
            .mul_element_wise(1.5);

        let mut new_position = self.pan_camera.position() + delta;
        new_position.x = new_position.x.clamp(-half_outer_size.x, half_outer_size.x);
        new_position.y = new_position.y.clamp(-half_outer_size.y, half_outer_size.y);
        self.pan_camera.set_position(new_position);
    }

    pub fn scale_view(&mut self, delta: f32) {
        const SCALE_SPEED: f32 = 100.0; // TODO: Make this customizable
        self.pan_camera.scale(delta * SCALE_SPEED);
    }

    pub fn selected_layer(&self) -> &Layer {
        self.document.current_layer()
    }

    pub fn assets(&self) -> Ref<AssetsLibrary> {
        self.assets.borrow()
    }

    pub fn camera(&self) -> &Camera2d {
        &self.pan_camera
    }
    pub fn camera_mut(&mut self) -> &mut Camera2d<'framework> {
        &mut self.pan_camera
    }
}
