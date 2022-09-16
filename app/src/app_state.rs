use std::{cell::RefCell, rc::Rc};

use framework::render_pass::RenderPass;
use framework::AssetsLibrary;
use framework::{Debug, Framework};
use image_editor::{ImageEditor, RenderToCanvasPass};
use log::info;
use wgpu::{
    CommandBuffer, CommandEncoderDescriptor, RenderPassColorAttachment, RenderPassDescriptor,
    Surface, SurfaceConfiguration, TextureView,
};
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::window::Window;

use crate::input_state::InputState;
use crate::toolbox::Toolbox;
use crate::tools::brush_engine::stamping_engine::StrokingEngine;
use crate::tools::{BrushTool, ColorPicker, HandTool};
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

    last_render: RenderToCanvasPass,
    stamping_engine: Rc<RefCell<StrokingEngine>>,
    brush_tool: Rc<RefCell<BrushTool<'framework>>>,
    hand_tool: Rc<RefCell<HandTool>>,
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

        let debug = Rc::new(RefCell::new(Debug::new()));

        let test_stamp = Toolbox::create_test_stamp();
        let stamping_engine = StrokingEngine::new(test_stamp, assets.clone());
        let stamping_engine = Rc::new(RefCell::new(stamping_engine));
        let brush_tool = Rc::new(RefCell::new(BrushTool::new(stamping_engine.clone(), 1.0)));
        let hand_tool = Rc::new(RefCell::new(HandTool::new()));
        let color_picker = Rc::new(RefCell::new(ColorPicker::new(stamping_engine.clone())));

        let (mut toolbox, brush_id, hand_id) =
            Toolbox::new(framework, brush_tool.clone(), hand_tool.clone());
        toolbox.add_tool(color_picker.clone());
        let ui = ui::create_ui(&framework, &final_surface_configuration, &window);
        let last_render =
            RenderToCanvasPass::new(&framework, wgpu::TextureFormat::Bgra8Unorm, assets.clone());
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

            stamping_engine,
            brush_tool,
            hand_tool,
            last_render,
        }
    }

    pub(crate) fn on_resized(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
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
                    stamping_engine: self.stamping_engine.clone(),
                    brush_tool: self.brush_tool.clone(),
                };
                let ui_handled_event = self.ui.do_ui(ui_ctx);

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

                let app_surface_view = current_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                self.image_editor.redraw_full_image();
                {
                    self.image_editor.render_canvas(&app_surface_view);
                }

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

        {}
        command_encoder.finish()
    }
}
