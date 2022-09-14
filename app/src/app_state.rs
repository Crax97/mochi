use std::{cell::RefCell, rc::Rc};

use framework::render_pass::RenderPass;
use framework::AssetsLibrary;
use framework::{Debug, Framework};
use image_editor::stamping_engine::StrokingEngine;
use image_editor::ImageEditor;
use log::info;
use wgpu::{
    CommandBuffer, CommandEncoderDescriptor, RenderPassColorAttachment, RenderPassDescriptor,
    Surface, SurfaceConfiguration, TextureView,
};
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::window::Window;

use crate::final_present_pass::FinalRenderPass;
use crate::input_state::InputState;
use crate::toolbox::Toolbox;
use crate::ui::{self, Ui, UiContext};

pub struct ImageApplication<'framework> {
    pub(crate) framework: &'framework Framework,
    pub(crate) window: Window,
    pub(crate) final_surface: Surface,
    pub(crate) final_surface_configuration: SurfaceConfiguration,
    pub(crate) debug: Rc<RefCell<Debug>>,
    image_editor: ImageEditor<'framework>,
    input_state: InputState,
    toolbox: Toolbox<'framework>,
    ui: Box<dyn Ui>,
    final_pass: FinalRenderPass<'framework>,
}
impl<'framework> ImageApplication<'framework> {
    pub(crate) fn new(window: Window, framework: &'framework Framework) -> Self {
        let final_surface = unsafe { framework.instance.create_surface(&window) };
        let final_surface_configuration = SurfaceConfiguration {
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: final_surface.get_supported_formats(&framework.adapter)[0],
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let assets = AssetsLibrary::new(framework);

        let image_editor = ImageEditor::new(&framework, assets.clone(), &[1024.0, 1024.0]);
        final_surface.configure(&framework.device, &final_surface_configuration);
        let final_present_pass = FinalRenderPass::new(
            framework,
            final_surface_configuration.clone(),
            &image_editor.get_full_image_texture(),
            assets.clone(),
        );

        let debug = Rc::new(RefCell::new(Debug::new()));

        let test_stamp = Toolbox::create_test_stamp(image_editor.camera().buffer(), framework);
        let stamping_engine = StrokingEngine::new(test_stamp, framework, assets.clone());
        let stamping_engine = Rc::new(RefCell::new(stamping_engine));

        let toolbox = Toolbox::new(framework, stamping_engine.clone());
        let ui = ui::create_ui(&framework, &final_surface_configuration, &window);
        Self {
            window,
            framework,
            final_surface,
            final_surface_configuration,
            debug,
            image_editor,
            input_state: InputState::default(),
            toolbox,
            ui: Box::new(ui),
            final_pass: final_present_pass,
        }
    }

    pub(crate) fn on_resized(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.final_pass
            .update_size([new_size.width as f32, new_size.height as f32]);
        let half_size = LogicalSize {
            width: new_size.width as f32 * 0.5,
            height: new_size.height as f32 * 0.5,
        };
        let left_right_top_bottom = [
            -half_size.width,
            half_size.width,
            half_size.height,
            -half_size.height,
        ];
        let new_surface_configuration = SurfaceConfiguration {
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self
                .final_surface
                .get_supported_formats(&self.framework.adapter)[0],
            width: new_size.width as u32,
            height: new_size.height as u32,
            present_mode: wgpu::PresentMode::Fifo,
        };
        self.final_surface
            .configure(&self.framework.device, &new_surface_configuration);
        self.final_surface_configuration = new_surface_configuration;
        self.image_editor.on_resize(left_right_top_bottom);
    }

    pub(crate) fn on_event(&mut self, event: &winit::event::Event<()>) -> ControlFlow {
        let debug = self.debug.clone();
        //debug.borrow_mut().begin_debug();

        self.input_state.update(&event);
        self.ui.on_new_winit_event(&event);
        self.toolbox
            .update(&self.input_state, &mut self.image_editor, debug.clone());

        match event {
            winit::event::Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        // if app.handle_on_close_requested() == AppFlow::Exit {
                        // *control_flow = ControlFlow::ExitWithCode(0);
                        // }
                        return ControlFlow::ExitWithCode(0);
                    }
                    WindowEvent::Resized(new_size) => {
                        self.on_resized(*new_size);
                    }
                    _ => {}
                }
            }
            winit::event::Event::UserEvent(_) => {}
            winit::event::Event::RedrawRequested(_) => {
                self.ui.begin();

                let ui_ctx = UiContext {
                    image_editor: &mut self.image_editor,
                    toolbox: &mut self.toolbox,
                    input_state: &self.input_state,
                };
                let ui_handled_event = self.ui.do_ui(ui_ctx);
                self.toolbox.set_enabled(!ui_handled_event);

                self.image_editor.update_layers();

                let current_texture = match self.final_surface.get_current_texture() {
                    Ok(surface) => surface,
                    Err(e) => match e {
                        wgpu::SurfaceError::Outdated => {
                            info!("RedrawRequested: early return because the current_texture is outdated (user resizing window maybe?)");
                            return ControlFlow::Wait;
                        }
                        _ => {
                            panic!("While presenting the last image: {e}");
                        }
                    },
                };

                let mut commands: Vec<CommandBuffer> = vec![];

                let draw_image_in_editor = { self.image_editor.redraw_full_image() };
                commands.push(draw_image_in_editor);

                let app_surface_view = current_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                /*
                               let debug_command = debug.borrow_mut().end_debug(
                                   &self.image_editor.get_full_image_texture().texture_view(),
                                   &self.assets,
                                   self.image_editor.camera().buffer(),
                                   &self.framework,
                               );
                               commands.push(debug_command);
                */
                let surface_configuration = self.final_surface_configuration.clone();

                let final_present_command = self.render_into_texture(&app_surface_view);
                commands.push(final_present_command);

                let ui_command = self.ui.present(
                    &self.framework,
                    &self.window,
                    surface_configuration,
                    &app_surface_view,
                );
                commands.push(ui_command);

                self.framework.queue.submit(commands);
                current_texture.present();
            }
            _ => {}
        }

        self.window.request_redraw();
        ControlFlow::Wait
    }

    fn render_into_texture(&self, current_texture: &TextureView) -> CommandBuffer {
        let command_encoder_description = CommandEncoderDescriptor {
            label: Some("Final image presentation"),
        };
        let mut command_encoder = self
            .framework
            .device
            .create_command_encoder(&command_encoder_description);

        let render_pass_description = RenderPassDescriptor {
            label: Some("ImageEditor present render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &current_texture,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 0.3,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        };

        {
            let render_pass = command_encoder.begin_render_pass(&render_pass_description);
            self.final_pass.execute_with_renderpass(render_pass, &[]);
        }
        command_encoder.finish()
    }
}
