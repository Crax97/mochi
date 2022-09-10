use std::{cell::RefCell, rc::Rc};

use framework::{mesh_names, AssetsLibrary};
use framework::{Debug, Framework, Mesh};
use image_editor::stamping_engine::StrokingEngine;
use image_editor::ImageEditor;
use log::info;
use wgpu::{
    BindGroup, ColorTargetState, CommandBuffer, CommandEncoderDescriptor, FragmentState,
    RenderPassColorAttachment, RenderPassDescriptor, Surface, SurfaceConfiguration, TextureView,
    VertexState,
};
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::{dpi::PhysicalSize, window::Window};

use crate::input_state::InputState;
use crate::toolbox::Toolbox;

pub(crate) struct ImageApplication<'framework> {
    pub(crate) framework: &'framework Framework,
    pub(crate) assets: Rc<AssetsLibrary>,
    pub(crate) window: Window,
    pub(crate) final_surface: Surface,
    pub(crate) final_surface_configuration: SurfaceConfiguration,
    pub(crate) debug: Rc<RefCell<Debug>>,
    image_editor: ImageEditor<'framework>,
    input_state: InputState,
    toolbox: Toolbox<'framework>,
    final_present_bind_group: BindGroup,
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
        final_surface.configure(&framework.device, &final_surface_configuration);

        let module = framework
            .device
            .create_shader_module(wgpu::include_wgsl!("shaders/final_present.wgsl"));

        let bind_group_layout =
            framework
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Final render group layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });
        let render_pipeline_layout =
            framework
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });
        let final_present_pipeline =
            framework
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("final render shader"),
                    layout: Some(&render_pipeline_layout),
                    depth_stencil: None,
                    vertex: VertexState {
                        module: &module,
                        entry_point: "vs",
                        buffers: &[Mesh::layout()],
                    },
                    fragment: Some(FragmentState {
                        module: &module,
                        entry_point: "fs",
                        targets: &[Some(ColorTargetState {
                            format: final_surface_configuration.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Cw,
                        conservative: false,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                    },
                });
        let mut library = AssetsLibrary::new(framework);
        library.add_pipeline(app_pipeline_names::FINAL_RENDER, final_present_pipeline);
        let assets = Rc::new(library);

        let debug = Rc::new(RefCell::new(Debug::new()));

        let image_editor = ImageEditor::new(
            &framework,
            assets.clone(),
            &[
                final_surface_configuration.width as f32,
                final_surface_configuration.height as f32,
            ],
        );

        let test_stamp = Toolbox::create_test_stamp(image_editor.camera().buffer(), framework);
        let stamping_engine = StrokingEngine::new(test_stamp, framework);
        let stamping_engine = Rc::new(RefCell::new(stamping_engine));
        let final_render = image_editor.get_full_image_texture();
        let bind_group = framework
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Final Draw render pass"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(final_render.texture_view()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(final_render.sampler()),
                    },
                ],
            });
        let toolbox = Toolbox::new(framework, stamping_engine.clone());
        Self {
            window,
            assets: assets.clone(),
            framework,
            final_surface,
            final_surface_configuration,
            debug,
            image_editor,
            input_state: InputState::default(),
            toolbox,
            final_present_bind_group: bind_group,
        }
    }

    pub(crate) fn on_resized(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        let half_size = PhysicalSize {
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
            width: new_size.width,
            height: new_size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        self.final_surface
            .configure(&self.framework.device, &new_surface_configuration);
        self.image_editor.on_resize(left_right_top_bottom);
    }

    pub(crate) fn on_event(&mut self, event: &winit::event::Event<()>) -> ControlFlow {
        self.input_state.update(&event);
        let debug = self.debug.clone();
        debug.borrow_mut().begin_debug();
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
            winit::event::Event::DeviceEvent { event, .. } => match event {
                _ => {
                    self.window.request_redraw();
                }
            },
            winit::event::Event::UserEvent(_) => {}
            winit::event::Event::RedrawRequested(_) => {
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

                let debug_command = debug.borrow_mut().end_debug(
                    &self.image_editor.get_full_image_texture().texture_view(),
                    &self.assets,
                    self.image_editor.camera().buffer(),
                    &self.framework,
                );
                commands.push(debug_command);

                let final_present_command = self.render_into_texture(&app_surface_view);
                commands.push(final_present_command);

                self.framework.queue.submit(commands);
                current_texture.present();
            }
            _ => {}
        }

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
            let mut render_pass = command_encoder.begin_render_pass(&render_pass_description);
            render_pass.set_pipeline(&self.assets.pipeline(app_pipeline_names::FINAL_RENDER));
            render_pass.set_bind_group(0, &self.final_present_bind_group, &[]);
            self.assets.mesh(mesh_names::QUAD).draw(&mut render_pass, 1);
        }
        command_encoder.finish()
    }
}

pub mod app_pipeline_names {
    pub const FINAL_RENDER: &'static str = "FINAL_RENDER";
}
