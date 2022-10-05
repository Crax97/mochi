use cgmath::{point2, ElementWise};
use framework::{Framework, MeshInstance2D};
use renderer::render_pass::texture2d_draw_pass::Texture2dDrawPass;
use scene::Camera2d;
use wgpu::TextureView;

use crate::document::DocumentCreationInfo;

use super::{
    document::Document,
    layers::{BitmapLayer, BitmapLayerConfiguration, Layer, LayerIndex},
};

#[derive(Default)]
pub struct LayerConstructionInfo {
    pub initial_color: [f32; 4],
    pub name: String,
}

pub struct ImageEditor<'framework> {
    framework: &'framework Framework,
    pan_camera: Camera2d,

    document: Document<'framework>,
    canvas: BitmapLayer,
}
impl<'framework> ImageEditor<'framework> {
    pub fn new(framework: &'framework Framework, initial_window_bounds: &[f32; 2]) -> Self {
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
        println!("Initial scale: {initial_camera_scale}");
        //pan_camera.set_scale(initial_camera_scale);

        let canvas = BitmapLayer::new(
            framework,
            BitmapLayerConfiguration {
                label: "ImageEditor Canvas".to_owned(),
                width: test_document.document_size().x,
                height: test_document.document_size().y,
                initial_background_color: [0.5, 0.5, 0.5, 1.0],
            },
        );
        ImageEditor {
            framework,
            canvas,
            pan_camera,
            document: test_document,
        }
    }

    pub fn framework(&'framework self) -> &'framework Framework {
        self.framework
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn mutate_document<F: FnMut(&mut Document)>(&mut self, mut mutate_fn: F) {
        mutate_fn(&mut self.document);
    }

    pub fn add_layer_to_document(&mut self, config: LayerConstructionInfo) {
        self.document.add_layer(self.framework, config);
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
        self.mutate_document(|d| d.update_layers());
    }

    pub fn render_document<'s, 't>(&'s mut self, mut pass: &mut Texture2dDrawPass<'framework>)
    where
        'framework: 't,
    {
        self.document.render(&mut pass);
    }

    pub fn render_canvas(&mut self, output_canvas: &TextureView, pass: &mut Texture2dDrawPass) {
        pass.begin(&self.camera());
        pass.draw_texture(
            self.document.final_layer(),
            MeshInstance2D::new(
                point2(0.0, 0.0),
                self.document.document_size().cast::<f32>().unwrap() * 0.5,
                0.0,
                false,
                1.0,
            ),
        );
        pass.execute(&self.framework, &output_canvas, true);
    }

    pub fn get_full_image_texture(&self) -> &BitmapLayer {
        &self.canvas
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
