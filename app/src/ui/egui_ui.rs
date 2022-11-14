use bytemuck::Zeroable;
use egui::{color::Hsva, Align2, Color32, FontDefinitions, Label, Pos2, RichText, Sense, Vec2};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::PlatformDescriptor;
use framework::Framework;
use image_editor::{
    blend_settings::BlendMode,
    layers::{LayerIndex, LayerSettings},
    LayerConstructionInfo,
};
use log::warn;
use strum::IntoEnumIterator;
use wgpu::{CommandBuffer, SurfaceConfiguration, TextureView};
use winit::window::Window;

use crate::{
    toolbox::ToolId,
    tools::{DynamicToolUi, EditorContext, Tool},
};

use super::{ToolUiContext, Ui, UiContext};
enum LayerAction {
    NewLayerRequest,
    CancelNewLayerRequest,
    CreateNewLayer,
    DeleteLayer(LayerIndex),
    SelectLayer(LayerIndex),
    SetLayerSettings(LayerIndex, LayerSettings),
    SelectNewTool(ToolId),
    None,
}

pub struct EguiUI {
    platform: egui_winit_platform::Platform,
    backend_pass: RenderPass,

    new_layer_in_creation: Option<LayerConstructionInfo>,
}

pub struct DynamicEguiUi<'a> {
    ui: &'a mut egui::Ui,
}
impl<'a> DynamicEguiUi<'a> {
    fn new(ui: &'a mut egui::Ui) -> Self {
        Self { ui }
    }
}

impl<'a> DynamicToolUi for DynamicEguiUi<'a> {
    fn label(&mut self, contents: &str) {
        self.ui.label(contents);
    }

    fn dropdown(
        &mut self,
        label: &str,
        current: usize,
        values_fn: Box<dyn FnOnce() -> Vec<(usize, String)>>,
    ) -> usize {
        let values = values_fn();
        let current = values
            .iter()
            .find(|(v, _)| *v == current)
            .expect("Current value is not contained in valid values!");
        let mut usize_number = current.0.clone();

        egui::ComboBox::from_label(label)
            .selected_text(current.1.clone())
            .show_ui(self.ui, |ui| {
                for (value, name) in values.iter() {
                    ui.selectable_value(&mut usize_number, *value, name);
                }
            });
        usize_number
    }
}

impl EguiUI {
    pub(crate) fn new(
        surface_configuration: &SurfaceConfiguration,
        window: &Window,
        framework: &Framework,
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

    fn do_ui_impl(&mut self, mut app_ctx: &mut UiContext) -> (bool, LayerAction) {
        if self.new_layer_in_creation.is_some() {
            self.new_layer_dialog()
        } else {
            let mut layer_action = LayerAction::None;
            let ctx = self.platform.context();
            let mut windows = vec![];
            windows.push(
                egui::Window::new("")
                    .anchor(Align2::LEFT_CENTER, Vec2::zeroed())
                    .show(&ctx, |ui| {
                        self.brush_settings(&mut app_ctx, ui);
                        let (_, action) = self.layer_settings(&mut app_ctx, ui);
                        layer_action = action;
                    })
                    .unwrap(),
            );
            windows.push(
                egui::Window::new("Tools")
                    .show(&ctx, |ui| {
                        egui::menu::bar(ui, |ui| {
                            egui::menu::menu_button(ui, "File", |ui| {});
                            egui::menu::menu_button(ui, "Edit", |ui| {
                                if ui.button("Selection to new layer").clicked() {
                                    app_ctx.image_editor.mutate_document(|doc| {
                                        doc.copy_layer_selection_to_new_layer(
                                            app_ctx.renderer,
                                            app_ctx.framework,
                                        );
                                        doc.mutate_selection(|sel| sel.clear());
                                    });
                                }
                                if ui.button("Invert selection").clicked() {
                                    app_ctx.image_editor.mutate_document(|doc| {
                                        doc.mutate_selection(|sel| sel.invert());
                                    });
                                }
                            });
                        });
                        ui.horizontal(|ui| {
                            app_ctx.toolbox.for_each_tool(|id, tool| {
                                let button = egui::Button::new(tool.name());
                                if ui
                                    .add_enabled(id != app_ctx.toolbox.primary_tool_id(), button)
                                    .clicked()
                                {
                                    layer_action = LayerAction::SelectNewTool(id.clone());
                                }
                            });

                            ui.separator();

                            let undo = egui::Button::new("Undo");
                            if ui
                                .add_enabled(app_ctx.undo_stack.has_undo(), undo)
                                .clicked()
                            {
                                app_ctx.undo_stack.do_undo(&mut EditorContext {
                                    framework: app_ctx.framework,
                                    image_editor: app_ctx.image_editor,
                                    renderer: app_ctx.renderer,
                                })
                            }
                            let redo = egui::Button::new("Redo");
                            if ui
                                .add_enabled(app_ctx.undo_stack.has_redo(), redo)
                                .clicked()
                            {
                                app_ctx.undo_stack.do_redo(&mut EditorContext {
                                    framework: app_ctx.framework,
                                    image_editor: app_ctx.image_editor,
                                    renderer: app_ctx.renderer,
                                })
                            }
                        });
                    })
                    .unwrap(),
            );

            let window_hovered = windows.iter().any(|win| {
                win.response.rect.contains(Pos2 {
                    x: app_ctx.input_state.mouse_position().x,
                    y: app_ctx.input_state.window_size().y as f32
                        - app_ctx.input_state.mouse_position().y,
                })
            });
            (window_hovered, layer_action)
        }
    }
}

impl EguiUI {
    fn brush_settings(&mut self, app_ctx: &mut UiContext, ui: &mut egui::Ui) -> bool {
        ui.label(egui::RichText::new("Brush").heading());
        let event_handled = false;
        let mut stamping_engine = app_ctx.stamping_engine.borrow_mut();
        let engine_config = stamping_engine.settings();
        let mut new_config = engine_config.clone();

        ui.horizontal(|ui| {
            ui.label("Brush color");
            use egui::color_picker::{color_picker_hsva_2d, Alpha};
            let mut hsva = Hsva::from_srgba_premultiplied([
                new_config.color_srgb[0],
                new_config.color_srgb[1],
                new_config.color_srgb[2],
                new_config.opacity,
            ]);
            if color_picker_hsva_2d(ui, &mut hsva, Alpha::Opaque) {
                new_config.color_srgb = hsva.to_srgb();
            }
        });
        ui.horizontal(|ui| {
            ui.label("Brush opacity");
            ui.add(
                egui::Slider::new(&mut new_config.opacity, 0..=255)
                    .custom_formatter(|n, _| format!("{:.2}", n / 255.0)),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Brush smoothness");
            ui.add(egui::Slider::new(&mut new_config.softness, 0.0..=10.0));
        });

        ui.horizontal(|ui| {
            ui.add(egui::Checkbox::new(&mut new_config.is_eraser, "Eraser"));
        });

        if new_config != engine_config {
            stamping_engine.set_new_settings(new_config);
        }

        let mut brush_tool = app_ctx.brush_tool.borrow_mut();
        ui.separator();
        ui.label("Brush tool settings");
        ui.horizontal(|ui| {
            ui.label("Brush size");
            ui.add(egui::DragValue::new(&mut brush_tool.size).clamp_range(1.0..=1000.0));
        });
        ui.horizontal(|ui| {
            ui.label("Pressure delta");
            ui.add(egui::DragValue::new(&mut brush_tool.pressure_delta).clamp_range(0.0..=1000.0));
        });
        ui.horizontal(|ui| {
            ui.label("Step");
            ui.add(egui::DragValue::new(&mut brush_tool.step).clamp_range(1.0..=1000.0));
        });

        if ui.button("Save").clicked() {
            app_ctx
                .image_editor
                .export_current_image(&app_ctx.framework);
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
            let layer = document.get_layer(idx);
            let original_settings = layer.settings();

            ui.push_id(&layer.uuid(), |ui| {
                let color = if *idx == document.current_layer_index() {
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
                egui::ComboBox::from_label("Blend mode")
                    .selected_text(format!("{:?}", settings.blend_mode))
                    .show_ui(ui, |ui| {
                        for key in BlendMode::iter() {
                            ui.selectable_value(&mut settings.blend_mode, key, key.to_string());
                        }
                    });

                if &settings != original_settings {
                    action = LayerAction::SetLayerSettings(idx.clone(), settings);
                }
            })
        };

        document.for_each_layer(|_, idx| {
            lay_layer_ui(idx);
        });

        (false, action)
    }

    fn new_layer_dialog(&mut self) -> (bool, LayerAction) {
        let ctx = self.platform.context();
        let mut action = LayerAction::None;
        let _ = egui::Window::new("Create new layer")
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(&ctx, |ui| {
                let layer_settings = self.new_layer_in_creation.as_mut().unwrap();

                ui.label("Layer color?");
                ui.color_edit_button_srgba_unmultiplied(&mut layer_settings.initial_color);
                ui.label("Layer name?");
                ui.text_edit_singleline(&mut layer_settings.name);
                if layer_settings.name.is_empty() {
                    egui::containers::show_tooltip(&ctx, egui::Id::new("invalid-layer"), |ui| {
                        ui.label("Cannot create a layer with an empty name!");
                    });
                }
                ui.horizontal(|ui| {
                    ui.label("Layer width");
                    ui.add(egui::DragValue::new(&mut layer_settings.width));
                    layer_settings.width = layer_settings.width.max(10);
                    ui.label("Layer height");
                    ui.add(egui::DragValue::new(&mut layer_settings.height));
                    layer_settings.height = layer_settings.height.max(10);
                });
                if ui.button("Create").clicked() && !layer_settings.name.is_empty() {
                    action = LayerAction::CreateNewLayer
                } else if ui.button("Cancel").clicked() {
                    action = LayerAction::CancelNewLayerRequest
                } else {
                    action = LayerAction::None
                }
            })
            .unwrap();
        return (true, action);
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
        let (block_editor, layer_action) = self.do_ui_impl(&mut app_ctx);
        match layer_action {
            LayerAction::NewLayerRequest => {
                self.new_layer_in_creation = Some(LayerConstructionInfo {
                    name: "New Layer".to_owned(),
                    width: 1024,
                    height: 1024,
                    ..Default::default()
                });
            }
            LayerAction::CancelNewLayerRequest => {
                self.new_layer_in_creation = None;
            }
            LayerAction::CreateNewLayer => {
                app_ctx.image_editor.add_layer_to_document(
                    self.new_layer_in_creation.take().unwrap(),
                    app_ctx.framework,
                );
            }
            LayerAction::DeleteLayer(idx) => app_ctx.image_editor.delete_layer(idx),
            LayerAction::SelectLayer(idx) => app_ctx.image_editor.select_new_layer(idx),
            LayerAction::SetLayerSettings(idx, settings) => {
                app_ctx.image_editor.mutate_document(|d| {
                    d.mutate_layer(&idx, |l| l.set_settings(settings.clone()));
                });
            }
            LayerAction::SelectNewTool(new_tool_id) => {
                app_ctx.toolbox.set_primary_tool(&new_tool_id);
            }
            LayerAction::None => {}
        };
        block_editor
    }
    fn do_tool_ui(&mut self, mut app_ctx: ToolUiContext, tool: &mut dyn Tool) -> bool {
        let ctx = self.platform.context();
        let window = egui::Window::new(tool.name()).show(&ctx, |ui| {
            let mut dynamic_ui = DynamicEguiUi::new(ui);
            tool.ui(&mut dynamic_ui);
            // dynamic_ui.do_stuff();
        });
        if let Some(response) = window {
            response.response.rect.contains(Pos2 {
                x: app_ctx.input_state.mouse_position().x,
                y: app_ctx.input_state.window_size().y as f32
                    - app_ctx.input_state.mouse_position().y,
            })
        } else {
            false
        }
    }
    fn present(
        &mut self,
        window: &Window,
        surface_configuration: SurfaceConfiguration,
        output_view: &TextureView,
        framework: &Framework,
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

    fn slider_formatted<N: cgmath::num_traits::Num, F: FnOnce(&N) -> String>(
        &mut self,
        current: &mut N,
        range: std::ops::RangeInclusive<N>,
        formatter: F,
    ) {
        todo!()
    }
}
