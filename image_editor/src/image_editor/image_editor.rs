use cgmath::{ElementWise, Point2};
use framework::framework::ShaderId;
use framework::renderer::draw_command::{DrawCommand, DrawMode, OptionalDrawData, PrimitiveType};
use framework::renderer::renderer::Renderer;
use framework::scene::Camera2d;
use framework::shader::{BindElement, ShaderCreationInfo};
use framework::{Framework, Transform2d};
use wgpu::{TextureFormat, TextureView};

use crate::document::DocumentCreationInfo;
use crate::image_editor;

use super::{
    document::Document,
    layers::{BitmapLayer, Layer, LayerIndex},
};

#[derive(Default)]
pub struct LayerConstructionInfo {
    pub initial_color: [f32; 4],
    pub name: String,
    pub width: u32,
    pub height: u32,
}

pub struct ImageEditor {
    pan_camera: Camera2d,
    layer_draw_shader: ShaderId,

    document: Document,
    final_present_shader: ShaderId,
}

impl ImageEditor {
    pub fn new(framework: &Framework, initial_window_bounds: &[f32; 2]) -> Self {
        image_editor::init_globals(framework);

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
        let pan_camera = Camera2d::new(-0.1, 1000.0, left_right_top_bottom);
        let initial_camera_scale = if initial_window_bounds[0] > initial_window_bounds[1] {
            test_document.outer_size().x / initial_window_bounds[0]
        } else {
            test_document.outer_size().y / initial_window_bounds[1]
        } * 1.5;

        let final_present_shader_info = ShaderCreationInfo::using_default_vertex_fragment()
            .with_output_format(TextureFormat::Bgra8UnormSrgb);
        let final_present_shader =
            framework::instance_mut().create_shader(final_present_shader_info);
        println!("Initial scale: {initial_camera_scale}");
        //pan_camera.set_scale(initial_camera_scale);

        let layer_draw_shader = framework::instance()
            .shader_compiler
            .compile_into_shader_description(
                "Layer draw shader",
                include_str!("layers/layer_fragment.wgsl"),
            )
            .unwrap();
        let fucking_shader_info = ShaderCreationInfo::using_default_vertex(layer_draw_shader)
            .with_bind_element(BindElement::Texture) // Bottom layer
            .with_bind_element(BindElement::Texture) // Top layer
            .with_bind_element(BindElement::UniformBuffer); // Blend settings
        let layer_draw_shader = framework::instance_mut().create_shader(fucking_shader_info);
        ImageEditor {
            pan_camera,
            document: test_document,
            final_present_shader,
            layer_draw_shader,
        }
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn export_current_image(&mut self) {
        let file_path = rfd::FileDialog::new()
            .add_filter("PNG Image", &["png"])
            .add_filter("JPG Image", &["jpg", "jpeg"])
            .add_filter("Bitmap", &["bmp"])
            .set_title("Save image")
            .save_file();
        if let Some(file_path) = file_path {
            let image = self.get_full_image_bytes();
            if let Err(e) = image.save(file_path) {
                log::error!("While saving image: {e}");
            };
        }
    }

    pub fn mutate_document<F: FnMut(&mut Document)>(&mut self, mut mutate_fn: F) {
        mutate_fn(&mut self.document);
    }

    pub fn add_layer_to_document(&mut self, config: LayerConstructionInfo) -> LayerIndex {
        self.document.add_layer(config)
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

    pub fn update_layers(&mut self, renderer: &mut Renderer) {
        self.mutate_document(|d| d.update_layers(renderer));
    }

    pub fn render_document(&mut self, renderer: &mut Renderer) {
        self.document
            .render(renderer, self.layer_draw_shader.clone());
    }

    pub fn render_canvas(&mut self, renderer: &mut Renderer, output_canvas: &TextureView) {
        renderer.begin(&self.pan_camera, Some(wgpu::Color::TRANSPARENT));
        renderer.draw(DrawCommand {
            primitives: PrimitiveType::Texture2D {
                texture_id: self.document.final_layer().texture().clone(),
                instances: vec![Transform2d {
                    scale: self.document.document_size().cast::<f32>().unwrap() * 0.5,
                    ..Default::default()
                }],
                flip_uv_y: true,
                multiply_color: wgpu::Color::WHITE,
            },
            draw_mode: DrawMode::Single,
            additional_data: OptionalDrawData {
                additional_vertex_buffers: vec![],
                additional_bindable_resource: vec![],
                shader: Some(self.final_present_shader.clone()),
            },
        });

        renderer.end(output_canvas, None);
    }

    pub fn get_full_image_texture(&self) -> &BitmapLayer {
        &self.document().final_layer()
    }

    pub fn get_full_image_bytes(&mut self) -> image::DynamicImage {
        self.document().final_image_bytes()
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

    // Transforms according to current camera position and current layer transform
    pub fn transform_point_into_pixel_position(
        &self,
        point_normalized: Point2<f32>,
    ) -> Option<Point2<f32>> {
        let position_into_layer = self.camera().ndc_into_world(point_normalized);
        Some(position_into_layer)
    }

    pub fn selected_layer(&self) -> &Layer {
        self.document.current_layer()
    }

    pub fn camera(&self) -> &Camera2d {
        &self.pan_camera
    }
    pub fn camera_mut(&mut self) -> &mut Camera2d {
        &mut self.pan_camera
    }
}
