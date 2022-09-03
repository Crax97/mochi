mod framework;
mod image_editor;

use std::{cell::RefCell, rc::Rc};

use cgmath::{point2, point3};
use framework::Framework;
use framework::*;
use image_editor::*;

use wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, ColorTargetState,
    CommandBuffer, CommandEncoderDescriptor, FragmentState, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, SamplerBindingType, Surface, SurfaceConfiguration,
    TextureDimension, TextureView, VertexState,
};
use winit::{event::WindowEvent, event_loop::ControlFlow, window::Window};

struct AppState {
    framework: Rc<Framework>,
    assets: Rc<Assets>,
    window: Window,
    image_editor: ImageEditor,
    final_surface: Surface,
    final_surface_configuration: SurfaceConfiguration,
}
impl AppState {
    fn new(window: Window, framework: Rc<Framework>) -> Self {
        let final_surface = unsafe { framework.instance.create_surface(&window) };
        let final_surface_configuration = SurfaceConfiguration {
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: final_surface.get_supported_formats(&framework.adapter)[0],
            width: 800,
            height: 600,
            present_mode: wgpu::PresentMode::Fifo,
        };
        final_surface.configure(&framework.device, &final_surface_configuration);

        let quad_mesh_vertices = [
            Vertex {
                position: point3(-0.5, 0.5, 0.0),
                tex_coords: point2(0.0, 1.0),
            },
            Vertex {
                position: point3(0.5, 0.5, 0.0),
                tex_coords: point2(1.0, 1.0),
            },
            Vertex {
                position: point3(-0.5, -0.5, 0.0),
                tex_coords: point2(0.0, 0.0),
            },
            Vertex {
                position: point3(0.5, -0.5, 0.0),
                tex_coords: point2(1.0, 0.0),
            },
        ]
        .into();

        let indices = [0u16, 1, 2, 2, 1, 3].into();
        let quad_mesh = Mesh::new(
            &framework,
            MeshConstructionDetails {
                vertices: quad_mesh_vertices,
                indices,
                allow_editing: false,
            },
        );

        let module = framework
            .device
            .create_shader_module(wgpu::include_wgsl!("simple_shader.wgsl"));

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

        let simple_diffuse_pipeline =
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
                            format: wgpu::TextureFormat::Rgba8UnormSrgb,
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

        let assets = Rc::new(Assets {
            quad_mesh,
            simple_diffuse_pipeline,
            final_present_pipeline,
        });
        Self {
            window,
            assets: assets.clone(),
            framework: framework.clone(),
            image_editor: ImageEditor::new(framework, assets),
            final_surface,
            final_surface_configuration,
        }
    }
}

async fn run_app() -> anyhow::Result<()> {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Image editor")
        .build(&event_loop)?;
    let framework = Framework::new(&wgpu::DeviceDescriptor {
        label: Some("Image Editor framework"),
        features: wgpu::Features::empty(),
        limits: wgpu::Limits::downlevel_defaults(),
    })
    .await?;
    framework.log_info();

    let app_state = Rc::new(RefCell::new(AppState::new(window, Rc::new(framework))));

    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => {
                // if app.handle_on_close_requested() == AppFlow::Exit {
                // *control_flow = ControlFlow::ExitWithCode(0);
                // }
                *control_flow = ControlFlow::ExitWithCode(0);
            }
            _ => {}
        },
        winit::event::Event::DeviceEvent { event, .. } => match event {
            _ => {
                app_state.borrow().window.request_redraw();
            }
        },
        winit::event::Event::UserEvent(_) => {}
        winit::event::Event::RedrawRequested(_) => {
            let mut commands: Vec<CommandBuffer> = vec![];

            let command = {
                let mut app_state_mut = app_state.borrow_mut();
                let image_editor = &mut app_state_mut.image_editor;
                image_editor.redraw_full_image()
            };
            commands.push(command);

            let app_state = app_state.borrow();

            let current_texture = app_state.final_surface.get_current_texture().unwrap();
            let mut render_last_frame = || {
                let render_result = app_state.image_editor.get_full_image_texture();
                let app_surface_view = current_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let command_encoder_description = CommandEncoderDescriptor {
                    label: Some("Final image presentation"),
                };
                let render_pass_description = RenderPassDescriptor {
                    label: Some("ImageEditor present render pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &app_surface_view,
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
                let mut command_encoder = app_state
                    .framework
                    .device
                    .create_command_encoder(&command_encoder_description);

                {
                    let mut render_pass =
                        command_encoder.begin_render_pass(&render_pass_description);
                    render_pass.set_pipeline(&app_state.assets.final_present_pipeline);
                    render_pass.set_bind_group(0, render_result.binding_group(), &[]);
                    app_state.assets.quad_mesh.draw(&mut render_pass, 1);
                }
                let final_command = command_encoder.finish();
                commands.push(final_command);
            };
            render_last_frame();
            app_state.framework.queue.submit(commands);
            current_texture.present();
        }
        _ => {}
    });
}

fn main() {
    env_logger::init();
    match pollster::block_on(run_app()) {
        Ok(_) => {}
        Err(e) => {
            log::error!("While running app: {}", e);
        }
    }
}
