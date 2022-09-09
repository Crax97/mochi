use std::{collections::HashMap, rc::Rc};

use cgmath::{point2, vec2, vec3, vec4, ElementWise, Point2, Transform, Vector2};
use framework::{Framework, Mesh, MeshInstance2D, TypedBuffer};
use scene::Camera2d;
use wgpu::{
    ColorTargetState, CommandBuffer, CommandEncoderDescriptor, FragmentState,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, VertexState,
};

use crate::{asset_library::AssetsLibrary, PipelineNames};

use super::{
    document::Document,
    layers::{
        BitmapLayer, BitmapLayerConfiguration, Layer, LayerCreationInfo, LayerDrawContext,
        LayerIndex, LayerTree, RootLayer,
    },
};

pub struct ImageEditor<'framework> {
    framework: &'framework Framework,
    assets: Rc<AssetsLibrary>,
    pan_camera: Camera2d<'framework>,

    document: Document<'framework>,
}

impl<'framework> ImageEditor<'framework> {
    pub fn new(
        framework: &'framework Framework,
        assets: Rc<AssetsLibrary>,
        initial_window_bounds: &[f32; 2],
    ) -> Self {
        let final_layer = BitmapLayer::new(
            &framework,
            BitmapLayerConfiguration {
                label: "Final Rendering Layer".to_owned(),
                width: 800,
                height: 600,
                initial_background_color: [0.5, 0.5, 0.5, 1.0],
            },
        );
        let test_layer = BitmapLayer::new(
            &framework,
            BitmapLayerConfiguration {
                label: "Layer 0".to_owned(),
                width: 800,
                height: 600,
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
        } * 0.25;
        println!("Initial scale: {initial_camera_scale}");
        // pan_camera.set_scale(initial_camera_scale);

        let test_layer_index = LayerIndex(0);
        let test_document = Document {
            layers: HashMap::from_iter(std::iter::once((
                test_layer_index,
                Layer::new_bitmap(
                    test_layer,
                    LayerCreationInfo {
                        position: point2(0.0, 0.0),
                        scale: vec2(1.0, 1.0),
                        rotation_radians: 0.0,
                        camera_buffer: pan_camera.buffer(),
                    },
                    &framework,
                ),
            ))),
            tree_root: RootLayer(vec![LayerTree::SingleLayer(test_layer_index)]),
            final_layer,
            current_layer_index: test_layer_index,
        };

        ImageEditor {
            framework,
            assets,
            pan_camera,
            document: test_document,
        }
    }

    pub fn framework(&'framework self) -> &'framework Framework {
        self.framework
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
        let render_pass_description = RenderPassDescriptor {
            label: Some("ImageEditor Redraw Image Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: self.document.final_layer.texture_view(),
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
        let mut command_encoder = self
            .framework
            .device
            .create_command_encoder(&command_encoder_description);

        {
            let mut render_pass = command_encoder.begin_render_pass(&render_pass_description);
            render_pass.set_pipeline(&self.assets.pipeline(PipelineNames::SIMPLE_TEXTURED));

            let mut draw_context = LayerDrawContext {
                render_pass: &mut render_pass,
                assets: &self.assets,
            };

            for layer_node in self.document.tree_root.0.iter() {
                match layer_node {
                    LayerTree::SingleLayer(index) => {
                        let layer = self.document.layers.get(index).expect("Nonexistent layer");
                        layer.draw(&mut draw_context);
                    }
                    LayerTree::Group(indices) => {
                        for index in indices {
                            let layer = self.document.layers.get(index).expect("Nonexistent layer");
                            layer.draw(&mut draw_context);
                        }
                    }
                };
            }
        }
        command_encoder.finish()
    }

    pub fn get_full_image_texture(&self) -> &BitmapLayer {
        &self.document.final_layer
    }

    pub(crate) fn pan_camera(&mut self, delta: cgmath::Vector2<f32>) {
        const MULT: f32 = 0.05; // TODO Understand why this is necessary

        let delta = self.pan_camera.current_scale() * delta * MULT;
        let half_outer_size = self.document.outer_size() * 0.25;

        let mut new_position = self.pan_camera.position() + delta;
        new_position.x = new_position.x.clamp(-half_outer_size.x, half_outer_size.x);
        new_position.y = new_position.y.clamp(-half_outer_size.y, half_outer_size.y);
        self.pan_camera.set_position(new_position);
    }

    pub fn scale_view(&mut self, delta: f32) {
        const SCALE_SPEED: f32 = 10.0; // TODO: Make this customizable
        self.pan_camera.scale(delta * SCALE_SPEED);
    }

    pub(crate) fn selected_layer(&self) -> &Layer {
        self.document.current_layer()
    }

    pub(crate) fn assets(&self) -> &AssetsLibrary {
        &self.assets
    }

    pub fn camera(&self) -> &Camera2d {
        &self.pan_camera
    }
    pub fn camera_mut(&mut self) -> &mut Camera2d<'framework> {
        &mut self.pan_camera
    }
}
