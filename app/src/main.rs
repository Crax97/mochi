mod app_state;
pub mod input_state;
mod toolbox;
pub mod tools;
mod ui;

use app_state::ImageApplication;
use framework::Framework;
pub use input_state::key::*;
pub use input_state::*;
use tools::*;

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
            features: wgpu::Features::empty(),
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
    let app_state = Box::leak(Box::new(ImageApplication::new(window, framework)));

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
