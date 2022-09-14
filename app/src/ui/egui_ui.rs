use bytemuck::Zeroable;
use egui::{Align2, Color32, FontDefinitions, InnerResponse, Label, Pos2, RichText, Sense, Vec2};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::PlatformDescriptor;
use framework::Framework;
use image::{ImageBuffer, Rgba};
use image_editor::{
    layers::{LayerIndex, LayerSettings},
    LayerConstructionInfo,
};
use log::warn;
use wgpu::{CommandBuffer, SurfaceConfiguration, TextureView};
use winit::window::Window;

use super::{Ui, UiContext};

enum LayerAction {
    NewLayerRequest,
    CancelNewLayerRequest,
    CreateNewLayer,
    DeleteLayer(LayerIndex),
    SelectLayer(LayerIndex),
    SetLayerSettings(LayerIndex, LayerSettings),
    None,
}

pub struct EguiUI {
    platform: egui_winit_platform::Platform,
    backend_pass: RenderPass,

    new_layer_in_creation: Option<LayerConstructionInfo>,
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
            new_layer_in_creation: None,
        }
    }
}

impl EguiUI {
    fn brush_settings(&mut self, app_ctx: &mut UiContext, ui: &mut egui::Ui) -> bool {
        ui.label(egui::RichText::new("Brush").heading());
        let mut event_handled = false;
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

        if ui.button("save test").clicked() {
            let bytes = app_ctx.image_editor.get_full_image_bytes();
            if let Some(buffer) =
                ImageBuffer::<Rgba<u8>, _>::from_raw(bytes.width, bytes.height, bytes.bytes)
            {
                buffer
                    .save("image.png")
                    .unwrap_or_else(|_| println!("Failed to write img"));
            }
        }

        event_handled
    }

    fn layer_settings(
        &mut self,
        app_ctx: &mut UiContext,
        ui: &mut egui::Ui,
    ) -> (bool, LayerAction) {
        ui.separator();
        ui.label(egui::RichText::new("Layers").heading());
        let document = app_ctx.image_editor.document();
        use image_editor::layers::LayerTree::*;

        let mut action = LayerAction::None;

        let sense = if self.new_layer_in_creation.is_none() {
            Sense::click()
        } else {
            Sense {
                click: false,
                drag: false,
                focusable: false,
            }
        };

        if ui
            .add(egui::Button::new("New layer").sense(sense))
            .clicked()
        {
            action = LayerAction::NewLayerRequest;
        }

        let mut lay_layer_ui = |idx: &LayerIndex| {
            let original_settings = document.get_layer(idx).settings();
            let color = if *idx == document.current_layer_index {
                Color32::LIGHT_BLUE
            } else {
                Color32::WHITE
            };
            let mut settings = original_settings.clone();

            ui.horizontal(|ui| {
                if ui
                    .add(Label::new(RichText::from(&settings.name).color(color)).sense(sense))
                    .clicked()
                {
                    action = LayerAction::SelectLayer(idx.clone());
                }

                ui.add(egui::Checkbox::new(&mut settings.is_enabled, ""));

                if ui
                    .add(egui::Button::new("Delete layer").sense(sense))
                    .clicked()
                {
                    action = LayerAction::DeleteLayer(idx.clone());
                }
            });

            ui.add(egui::Slider::new(&mut settings.opacity, 0.0..=1.0).text("Opacity"));

            if settings != original_settings {
                action = LayerAction::SetLayerSettings(idx.clone(), settings);
            }
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

        (false, action)
    }

    fn new_layer_dialog(&mut self, app_ctx: &mut UiContext) -> LayerAction {
        let ctx = self.platform.context();
        let mut action = LayerAction::None;
        egui::Window::new("Create new layer")
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(&ctx, |ui| {
                let layer_settings = self.new_layer_in_creation.as_mut().unwrap();

                ui.label("Layer color?");
                ui.color_edit_button_rgba_unmultiplied(&mut layer_settings.initial_color);
                ui.label("Layer name?");
                ui.text_edit_singleline(&mut layer_settings.name);
                if layer_settings.name.is_empty() {
                    egui::containers::show_tooltip(&ctx, egui::Id::new("invalid-layer"), |ui| {
                        ui.label("Cannot create a layer with an empty name!");
                    });
                }
                if ui.button("Create").clicked() && !layer_settings.name.is_empty() {
                    action = LayerAction::CreateNewLayer
                } else if ui.button("Cancel").clicked() {
                    action = LayerAction::CancelNewLayerRequest
                } else {
                    action = LayerAction::None
                }
            });
        return action;
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
        if self.new_layer_in_creation.is_some() {
            let action = self.new_layer_dialog(&mut app_ctx);
            match action {
                LayerAction::NewLayerRequest => {
                    self.new_layer_in_creation = Some(LayerConstructionInfo::default());
                    return true;
                }
                LayerAction::CancelNewLayerRequest => {
                    self.new_layer_in_creation = None;
                    return false;
                }
                LayerAction::CreateNewLayer => {
                    app_ctx
                        .image_editor
                        .add_layer_to_document(self.new_layer_in_creation.take().unwrap());
                    return true;
                }
                _ => {
                    return false;
                }
            }
        } else {
            let ctx = self.platform.context();
            let mut res = false;
            egui::Window::new("")
                .anchor(Align2::LEFT_CENTER, Vec2::zeroed())
                .show(&ctx, |ui| {
                    res |= self.brush_settings(&mut app_ctx, ui);
                    let (hover_layer, layer_action) = self.layer_settings(&mut app_ctx, ui);
                    res |= hover_layer;

                    match layer_action {
                        LayerAction::NewLayerRequest => {
                            self.new_layer_in_creation = Some(LayerConstructionInfo::default());
                        }
                        LayerAction::DeleteLayer(idx) => app_ctx.image_editor.delete_layer(idx),
                        LayerAction::SelectLayer(idx) => app_ctx.image_editor.select_new_layer(idx),
                        LayerAction::SetLayerSettings(idx, settings) => {
                            let document = app_ctx.image_editor.mutate_document();
                            document.get_layer_mut(&idx).set_settings(settings);
                        }
                        _ => {}
                    };
                });
            res
        }
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
