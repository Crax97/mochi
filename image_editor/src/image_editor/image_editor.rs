use std::{
    cell::{Ref, RefCell},
    num::NonZeroU32,
    rc::Rc,
};

use cgmath::{point2, ElementWise};
use framework::{render_pass::RenderPass, Framework, TypedBuffer, TypedBufferConfiguration};
use image::Rgba;
use scene::Camera2d;
use wgpu::{
    BindGroup, CommandBuffer, CommandEncoder, CommandEncoderDescriptor, RenderPassColorAttachment,
    RenderPassDescriptor,
};

use framework::asset_library::AssetsLibrary;

use crate::{document::DocumentCreationInfo, layers::LayerDrawPass};

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
    layer_draw_pass: LayerDrawPass,

    camaera_bind_group: BindGroup,
}
impl<'framework> ImageEditor<'framework> {
    pub fn new(
        framework: &'framework Framework,
        assets: Rc<RefCell<AssetsLibrary>>,
        initial_window_bounds: &[f32; 2],
    ) -> Self {
        let test_width = 1800;
        let test_height = 1024;
        let test_document = Document::new(DocumentCreationInfo {
            width: test_width,
            height: test_height,
            first_layer_color: [0.0, 0.0, 0.0, 1.0],
        });
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
            pan_camera,
            document: test_document,
            layers_created: 0,
            layer_draw_pass,
            camaera_bind_group,
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
        self.render_canvas();
    }

    fn render_document(&mut self) {
        self.document.render();
    }

    fn render_canvas(&mut self) {}

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
