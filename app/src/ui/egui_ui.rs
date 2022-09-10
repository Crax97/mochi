use std::iter;

use egui::{Color32, FontDefinitions, InnerResponse, Pos2};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::PlatformDescriptor;
use framework::Framework;
use log::warn;
use wgpu::{CommandBuffer, SurfaceConfiguration, TextureView};
use winit::window::Window;

use crate::{app_state::ImageApplication, toolbox::Toolbox};

use super::{Ui, UiContext};

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

impl Ui for EguiUI {
    fn begin(&mut self) {
        self.platform.begin_frame();
    }
    fn on_new_winit_event(&mut self, event: &winit::event::Event<()>) {
        self.platform.handle_event(&event);
    }
    fn do_ui(&mut self, app_ctx: UiContext) -> bool {
        let ctx = self.platform.context();
        let mut event_handled = false;
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

            if new_config != engine_config {
                app_ctx.toolbox.update_stamping_engine_data(new_config);
            }

            ui.separator();
            ui.label("Brush tool settings");
            ui.horizontal(|ui| {
                ui.label("Step");
                ui.add(
                    egui::DragValue::new(&mut app_ctx.toolbox.brush_tool.step)
                        .clamp_range(1.0..=1000.0),
                );
            })
        });
        if let Some(InnerResponse { response, .. }) = window_handled {
            let mouse_pos = app_ctx.input_state.mouse_position();
            event_handled = response.rect.contains(Pos2::new(mouse_pos.x, mouse_pos.y));
        }
        event_handled
    }
    fn present(
        &mut self,
        framework: &Framework,
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
            scale_factor: 1.0,
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
