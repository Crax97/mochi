use egui::{Color32, FontDefinitions, InnerResponse, Label, Pos2, RichText, Sense};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::PlatformDescriptor;
use framework::Framework;
use image_editor::layers::LayerIndex;
use log::warn;
use wgpu::{CommandBuffer, SurfaceConfiguration, TextureView};
use winit::window::Window;

use super::{Ui, UiContext};

enum LayerAction {
    NewLayer,
    DeleteLayer(LayerIndex),
    SelectLayer(LayerIndex),
    None,
}

pub struct EguiUI {
    platform: egui_winit_platform::Platform,
    backend_pass: RenderPass,
}

impl EguiUI {
    pub(crate) fn new(
        framework: &framework::Framework,
        surface_configuration: &SurfaceConfiguration,
        window: &Window,
    ) -> Self {
        Self {
            platform: egui_winit_platform::Platform::new(PlatformDescriptor {
                physical_width: surface_configuration.width,
                physical_height: surface_configuration.height,
                scale_factor: window.scale_factor(),
                font_definitions: FontDefinitions::default(),
                style: Default::default(),
            }),
            backend_pass: RenderPass::new(&framework.device, surface_configuration.format, 1),
        }
    }
}

impl EguiUI {
    fn brush_settings(&mut self, app_ctx: &mut UiContext) -> bool {
        let mut event_handled = false;
        let ctx = self.platform.context();
        let window_handled = egui::Window::new("Brush settings").show(&ctx, |ui| {
            let engine_config = app_ctx.toolbox.stamping_engine().settings();
            let mut new_config = engine_config.clone();

            ui.horizontal(|ui| {
                ui.label("Brush color");
                ui.color_edit_button_rgba_premultiplied(&mut new_config.color);
            });

            ui.horizontal(|ui| {
                ui.label("Brush smoothness");
                ui.add(egui::Slider::new(&mut new_config.softness, 0.0..=1.0));
            });

            ui.horizontal(|ui| {
                ui.add(egui::Checkbox::new(&mut new_config.is_eraser, "Eraser"));
            });

            if new_config != engine_config {
                app_ctx.toolbox.update_stamping_engine_data(new_config);
            }

            ui.separator();
            ui.label("Brush tool settings");
            ui.horizontal(|ui| {
                let max = app_ctx.toolbox.brush_tool.max_size;
                ui.label("Min brush size");
                ui.add(
                    egui::DragValue::new(&mut app_ctx.toolbox.brush_tool.min_size)
                        .clamp_range(1.0..=max),
                );
            });
            ui.horizontal(|ui| {
                let min = app_ctx.toolbox.brush_tool.min_size;
                ui.label("Max brush size");
                ui.add(
                    egui::DragValue::new(&mut app_ctx.toolbox.brush_tool.max_size)
                        .clamp_range(min..=1000.0),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Step");
                ui.add(
                    egui::DragValue::new(&mut app_ctx.toolbox.brush_tool.step)
                        .clamp_range(1.0..=1000.0),
                );
            });
        });
        if let Some(InnerResponse { response, .. }) = window_handled {
            let mouse_pos = app_ctx.input_state.mouse_position();
            event_handled = response.rect.contains(Pos2::new(mouse_pos.x, mouse_pos.y));
        }
        event_handled
    }

    fn layer_settings(&mut self, app_ctx: &mut UiContext) -> (bool, LayerAction) {
        let ctx = self.platform.context();
        let document = app_ctx.image_editor.document();
        use image_editor::layers::LayerTree::*;

        let mut action = LayerAction::None;

        let window_handled = egui::Window::new("Layers").show(&ctx, |ui| {
            if ui.button("New layer").clicked() {
                action = LayerAction::NewLayer;
            }

            let mut lay_layer_ui = |idx: &LayerIndex| {
                ui.horizontal(|ui| {
                    let layer = document.get_layer(idx);
                    let color = if *idx == document.current_layer_index {
                        Color32::LIGHT_BLUE
                    } else {
                        Color32::WHITE
                    };
                    if ui
                        .add(
                            Label::new(RichText::from(&layer.name).color(color))
                                .sense(Sense::click()),
                        )
                        .clicked()
                    {
                        action = LayerAction::SelectLayer(idx.clone());
                    }

                    if ui.button("Delete layer").clicked() {
                        action = LayerAction::DeleteLayer(idx.clone());
                    }
                });
            };
            for layer in document.tree_root.0.iter() {
                match layer {
                    SingleLayer(idx) => {
                        lay_layer_ui(idx);
                    }
                    Group(indices) => {
                        for idx in indices.iter() {
                            lay_layer_ui(idx);
                        }
                    }
                }
            }
        });
        let hover = if let Some(InnerResponse { response, .. }) = window_handled {
            let mouse_pos = app_ctx.input_state.mouse_position();
            response.rect.contains(Pos2::new(mouse_pos.x, mouse_pos.y))
        } else {
            false
        };
        (hover, action)
    }
}

impl Ui for EguiUI {
    fn begin(&mut self) {
        self.platform.begin_frame();
    }
    fn on_new_winit_event(&mut self, event: &winit::event::Event<()>) {
        self.platform.handle_event(&event);
    }
    fn do_ui(&mut self, mut app_ctx: UiContext) -> bool {
        let brush = self.brush_settings(&mut app_ctx);
        let (hover_layer, layer_action) = self.layer_settings(&mut app_ctx);

        match layer_action {
            LayerAction::NewLayer => app_ctx.image_editor.add_layer_to_document(),
            LayerAction::DeleteLayer(idx) => app_ctx.image_editor.delete_layer(idx),
            LayerAction::SelectLayer(idx) => app_ctx.image_editor.select_new_layer(idx),
            LayerAction::None => {}
        }

        brush || hover_layer
    }
    fn present(
        &mut self,
        framework: &Framework,
        window: &Window,
        surface_configuration: SurfaceConfiguration,
        output_view: &TextureView,
    ) -> CommandBuffer {
        let output = self.platform.end_frame(None);
        let paint_jobs = self.platform.context().tessellate(output.shapes);
        let mut encoder =
            framework
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("egui Ui rendering"),
                });
        let screen_descriptor = ScreenDescriptor {
            physical_width: surface_configuration.width,
            physical_height: surface_configuration.height,
            scale_factor: window.scale_factor() as f32,
        };
        let tdelta: egui::TexturesDelta = output.textures_delta;
        self.backend_pass
            .add_textures(&framework.device, &framework.queue, &tdelta)
            .expect("add texture ok");
        self.backend_pass.update_buffers(
            &framework.device,
            &framework.queue,
            &paint_jobs,
            &screen_descriptor,
        );

        // Record all render passes.
        self.backend_pass
            .execute(
                &mut encoder,
                &output_view,
                &paint_jobs,
                &screen_descriptor,
                None,
            )
            .unwrap();

        if let Err(e) = self.backend_pass.remove_textures(tdelta) {
            warn!("While executing ui pass: {e}");
        }
        encoder.finish()
    }
}
