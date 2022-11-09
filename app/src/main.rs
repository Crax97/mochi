mod app_state;
pub mod input_state;
mod toolbox;
pub mod tools;
mod ui;

use app_state::ImageApplication;
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

    framework::setup_framework(&wgpu::DeviceDescriptor {
        label: Some("Image Editor framework"),
        features: wgpu::Features::empty(),
        limits: wgpu::Limits {
            max_bind_groups: 5,
            ..Default::default()
        },
    });
    framework::instance_mut()
        .shader_compiler
        .define("blend_modes", include_str!("blend_modes.wgsl"))
        .unwrap();
    let app_state = Box::leak(Box::new(ImageApplication::new(window)));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = app_state.on_event(&event);
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
