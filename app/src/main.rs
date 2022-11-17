mod app_loop;
mod app_state;
mod application;
pub mod input_state;
mod toolbox;
pub mod tools;
mod ui;

use app_state::ImageApplication;
use framework::Framework;
pub use input_state::key::*;
pub use input_state::*;
use tools::*;

use wgpu::SurfaceConfiguration;
use winit::dpi::PhysicalSize;

async fn run_app() -> anyhow::Result<()> {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Image editor")
        .with_min_inner_size(PhysicalSize {
            width: 800,
            height: 600,
        })
        .build(&event_loop)?;

    let framework = Box::leak(Box::new({
        let framework = Framework::new(&wgpu::DeviceDescriptor {
            label: Some("Image Editor framework"),
            features: wgpu::Features::DEPTH32FLOAT_STENCIL8,
            limits: wgpu::Limits {
                max_bind_groups: 5,
                ..Default::default()
            },
        });

        match framework {
            Ok(framework) => {
                framework.log_info();
                framework
            }
            Err(e) => {
                panic!("Error while creating framework: {}", e)
            }
        }
    }));
    framework
        .shader_compiler
        .define("blend_modes", include_str!("blend_modes.wgsl"))
        .unwrap();

    let final_surface = unsafe { framework.instance.create_surface(&window) };
    let final_surface_configuration = SurfaceConfiguration {
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: final_surface.get_supported_formats(&framework.adapter)[0],
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: wgpu::PresentMode::Immediate,
        alpha_mode: wgpu::CompositeAlphaMode::Opaque,
    };
    final_surface.configure(&framework.device, &final_surface_configuration);
    let ui = ui::create_ui(&final_surface_configuration, &window, &framework);

    let app_state = Box::leak(Box::new(ImageApplication::new(
        window,
        framework,
        ui,
        final_surface,
        final_surface_configuration,
    )));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = app_state.on_event(&event, framework);
    });
}

fn main() {
    env_logger::init();

    let result = pollster::block_on(run_app());
    match result {
        Ok(()) => {}
        Err(e) => panic!("Error while running application: {e}"),
    };
}
