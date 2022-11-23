use cgmath::{ElementWise, Point2};
use framework::framework::{ShaderId, TextureId};
use framework::renderer::draw_command::{DrawCommand, DrawMode, OptionalDrawData, PrimitiveType};
use framework::renderer::renderer::Renderer;
use framework::scene::Camera2d;
use framework::shader::{BindElement, ShaderCreationInfo};
use framework::{
    Framework, RgbaTexture2D, Texture, TextureConfiguration, TextureUsage, Transform2d,
};
use wgpu::{TextureFormat, TextureView};

use crate::document::DocumentCreationInfo;
use crate::image_editor;
use crate::layers::LayerId;

use super::{document::Document, layers::Layer};

#[derive(Default)]
pub struct LayerConstructionInfo {
    pub initial_color: [u8; 4],
    pub name: String,
}

pub struct ImageEditor {
    pan_camera: Camera2d,

    document: Document,
    output_texture: TextureId,
    final_present_shader: ShaderId,
}

impl ImageEditor {
    pub fn new(framework: &mut Framework, initial_window_bounds: &[f32; 2]) -> Self {
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

        let final_present_shader_info =
            ShaderCreationInfo::using_default_vertex_fragment(framework)
                .with_output_format(TextureFormat::Bgra8UnormSrgb);
        let final_present_shader = framework.create_shader(final_present_shader_info);

        let output_texture = framework.allocate_texture2d(
            RgbaTexture2D::empty((pan_camera.width() as u32, pan_camera.height() as u32)),
            TextureConfiguration {
                label: Some("ImageEditor final rendering texture"),
                usage: TextureUsage::RWRT,
                mip_count: None,
            },
        );

        ImageEditor {
            pan_camera,
            document: test_document,
            final_present_shader,
            output_texture,
        }
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn export_current_image(&mut self, framework: &Framework) {
        let file_path = rfd::FileDialog::new()
            .add_filter("PNG Image", &["png"])
            .add_filter("JPG Image", &["jpg", "jpeg"])
            .add_filter("Bitmap", &["bmp"])
            .set_title("Save image")
            .save_file();
        if let Some(file_path) = file_path {
            let image = self.get_full_image_bytes(framework);
            if let Err(e) = image.save(file_path) {
                log::error!("While saving image: {e}");
            };
        }
    }

    pub fn mutate_document<F: FnMut(&mut Document)>(&mut self, mut mutate_fn: F) {
        mutate_fn(&mut self.document);
    }

    pub fn add_layer_to_document(
        &mut self,
        config: LayerConstructionInfo,
        framework: &mut Framework,
    ) -> LayerId {
        self.document.add_layer(config, framework)
    }

    pub fn select_new_layer(&mut self, layer_idx: LayerId) {
        self.document.select_layer(layer_idx);
    }

    pub fn delete_layer(&mut self, layer_idx: LayerId) {
        self.document.delete_layer(layer_idx);
    }

    pub fn on_resize(&mut self, new_bounds: [f32; 4], framework: &mut Framework) {
        self.pan_camera.set_new_bounds(new_bounds);
        self.output_texture = framework.allocate_texture2d(
            RgbaTexture2D::empty((
                self.pan_camera.width() as u32,
                self.pan_camera.height() as u32,
            )),
            TextureConfiguration {
                label: Some("ImageEditor final rendering texture"),
                usage: TextureUsage::RWRT,
                mip_count: None,
            },
        );
    }

    pub fn update_layers(&mut self, renderer: &mut Renderer, framework: &mut Framework) {
        self.mutate_document(|d| d.update_layers(renderer, framework));
    }

    pub fn render_document(&mut self, renderer: &mut Renderer, framework: &mut Framework) {
        self.document.render(renderer, framework);
    }

    pub fn render_canvas(
        &mut self,
        renderer: &mut Renderer,
        output_canvas: &TextureView,
        framework: &mut Framework,
    ) {
        renderer.begin(&self.pan_camera, Some(wgpu::Color::TRANSPARENT), framework);
        renderer.set_draw_debug_name("Canvas rendering");
        renderer.draw(DrawCommand {
            primitives: PrimitiveType::Texture2D {
                texture_id: self.document.render_result().clone(),
                instances: vec![Transform2d {
                    scale: self.document.document_size().cast::<f32>().unwrap() * 0.5,
                    ..Default::default()
                }],
                flip_uv_y: true,
                multiply_color: wgpu::Color::WHITE,
            },
            draw_mode: DrawMode::Single,
            additional_data: OptionalDrawData::default(),
        });

        self.render_ui(renderer);

        renderer.end(&self.output_texture, None, framework);

        renderer.begin(&Camera2d::unit(), Some(wgpu::Color::TRANSPARENT), framework);
        renderer.draw(DrawCommand {
            primitives: PrimitiveType::Texture2D {
                texture_id: self.output_texture.clone(),
                instances: vec![Transform2d::default()],
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
        renderer.end_on_external_texture(output_canvas, framework);
    }

    fn render_ui(&mut self, renderer: &mut Renderer) {
        self.document.draw_selection(renderer);
    }

    pub fn get_full_image_texture(&self) -> &TextureId {
        &self.document().render_result()
    }

    pub fn get_full_image_bytes(&mut self, framework: &Framework) -> image::DynamicImage {
        self.document().final_image_bytes(framework)
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
        if self.pan_camera.current_scale() <= 0.0 {
            self.pan_camera.set_scale(0.01);
        }
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
