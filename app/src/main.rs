mod app_state;
mod input_state;

use app_state::{AppState, AppPipelineNames};
use cgmath::{Vector2, Point2, point2};
use framework::{Framework, TypedBuffer};
use image_editor::{*, stamping_engine::{StrokingEngine, Stamp, StampCreationInfo}, layers::{BitmapLayer, BitmapLayerConfiguration}};
use input_state::InputState;
use lazy_static::lazy_static;

use log::info;
use rand::Rng;
use wgpu::{
    BindGroup, CommandBuffer, CommandEncoderDescriptor, RenderPassColorAttachment,
    RenderPassDescriptor, SurfaceTexture,
};
use winit::{dpi::PhysicalSize, event::{WindowEvent, MouseButton}, event_loop::ControlFlow};

lazy_static! {
    static ref FRAMEWORK: Framework = pollster::block_on(async {
        let framework = Framework::new(&wgpu::DeviceDescriptor {
            label: Some("Image Editor framework"),
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::downlevel_defaults(),
        })
        .await;
        match framework {
            Ok(framework) => {
                framework.log_info();
                framework
            }
            Err(e) => {
                panic!("Error while creating framework: {}", e)
            }
        }
    });
}

async fn run_app() -> anyhow::Result<()> {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Image editor")
        .with_min_inner_size(PhysicalSize {
            width: 800,
            height: 600,
        })
        .build(&event_loop)?;

    let app_state = AppState::new(window, &FRAMEWORK);
    let mut input_state = InputState::default();
    let mut image_editor = ImageEditor::new(
        &FRAMEWORK,
        app_state.assets.clone(),
        &[
            app_state.final_surface_configuration.width as f32,
            app_state.final_surface_configuration.height as f32,
        ],
    );

    let bind_group_layout =
        app_state
            .framework
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
    let final_render = image_editor.get_full_image_texture();
    let bind_group = app_state
        .framework
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
        let test_stamp = create_test_stamp(image_editor.camera().buffer());
        let mut brush_tool = BrushTool::new(Box::new(StrokingEngine::new(test_stamp, &FRAMEWORK)), 5.0);
        let mut hand_tool = HandTool::new();

    event_loop.run(move |event, _, control_flow| {
        input_state.update(&event);
        match event {
        winit::event::Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => {
                // if app.handle_on_close_requested() == AppFlow::Exit {
                // *control_flow = ControlFlow::ExitWithCode(0);
                // }
                *control_flow = ControlFlow::ExitWithCode(0);
            }
            WindowEvent::Resized(new_size) => {
                app_state.on_resized(new_size, &mut image_editor);
            }
            _ => {}
        },
        winit::event::Event::DeviceEvent { event, .. } => match event {
            _ => {
                app_state.window.request_redraw();
            }
        },
        winit::event::Event::UserEvent(_) => {}
        winit::event::Event::RedrawRequested(_) => {
            image_editor.update_layers();
            let current_texture = match app_state.final_surface.get_current_texture() {
                Ok(surface) => surface,
                Err(e) => match e {
                    wgpu::SurfaceError::Outdated => {
                        info!("RedrawRequested: early return because the current_texture is outdated (user resizing window maybe?)");
                        return ;
                    }
                    _ => {
                        panic!("While presenting the last image: {e}");
                    }
                },
            };
            let mut commands: Vec<CommandBuffer> = vec![];

            let draw_image_in_editor = { image_editor.redraw_full_image() };
            commands.push(draw_image_in_editor);

            let final_present_command = render_into_texture(&current_texture, &app_state, &bind_group);
            commands.push(final_present_command);

            app_state.framework.queue.submit(commands);
            current_texture.present();
        }
        _ => {}
    }
        if input_state.is_mouse_button_just_pressed(MouseButton::Left) {
            brush_tool.on_pointer_click(PointerClick {pointer_location: input_state.normalized_mouse_position()}, EditorContext { image_editor: &mut image_editor });
        } else if input_state.is_mouse_button_just_released(MouseButton::Left) {
            brush_tool.on_pointer_release(PointerRelease {}, EditorContext { image_editor: &mut image_editor });
        } else {
            brush_tool.on_pointer_move(PointerMove {new_pointer_location: input_state.normalized_mouse_position(), delta_normalized: input_state.normalized_mouse_delta()}, EditorContext { image_editor: &mut image_editor });
        }
        if input_state.is_mouse_button_just_pressed(MouseButton::Right) {
            hand_tool.on_pointer_click(PointerClick {pointer_location: input_state.normalized_mouse_position()}, EditorContext { image_editor: &mut image_editor });
        } else if input_state.is_mouse_button_just_released(MouseButton::Right) {
            hand_tool.on_pointer_release(PointerRelease {}, EditorContext { image_editor: &mut image_editor });
        } else {
            hand_tool.on_pointer_move(PointerMove {new_pointer_location: input_state.normalized_mouse_position(), delta_normalized: input_state.normalized_mouse_delta()}, EditorContext { image_editor: &mut image_editor });
        }
        if input_state.is_mouse_button_just_pressed(MouseButton::Middle) {
            image_editor.camera_mut().set_position(point2(0.0, 0.0));
        }
        if input_state.mouse_wheel_delta().abs() > 0.0 {
            image_editor.scale_view(input_state.mouse_wheel_delta());
        }
    });
}

fn create_test_stamp(camera_buffer: &TypedBuffer) -> Stamp {
    let test_stamp_bytes = include_bytes!("test/test_brush.png");
    let image = image::load_from_memory(test_stamp_bytes).unwrap();
    let brush_bitmap = BitmapLayer::new_from_bytes(&FRAMEWORK, image.as_bytes(), BitmapLayerConfiguration {
        label: "Test brush".to_owned(),
        width: image.width(),
        initial_background_color: [0.0, 0.0, 0.0, 0.0],
        height: image.height(),
    });
    Stamp::new(brush_bitmap, &FRAMEWORK, StampCreationInfo {
        camera_buffer,
    })
}

fn render_into_texture(current_texture: &SurfaceTexture, app_state: &AppState, bind_group: &BindGroup) -> CommandBuffer {
    let command_encoder_description = CommandEncoderDescriptor {
        label: Some("Final image presentation"),
    };
    let mut command_encoder = app_state
        .framework
        .device
        .create_command_encoder(&command_encoder_description);


    let app_surface_view = current_texture
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
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

    {
        let mut render_pass = command_encoder.begin_render_pass(&render_pass_description);
        render_pass.set_pipeline(&app_state.assets.pipeline(AppPipelineNames::FINAL_RENDER));
        render_pass.set_bind_group(0, &bind_group, &[]);
        app_state.assets.mesh(MeshNames::QUAD).draw(&mut render_pass, 1);
    }
    command_encoder.finish()
}

fn main() {
    env_logger::init();

    let result = pollster::block_on(run_app());
    match result {
        Ok(()) => {}
        Err(e) => panic!("Error while running application: {e}"),
    };
}
