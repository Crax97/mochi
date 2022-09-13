mod app_state;
mod final_present_pass;
mod input_state;
mod toolbox;
mod ui;

use app_state::ImageApplication;
use final_present_pass::*;
use framework::Framework;
use lazy_static::lazy_static;

use winit::dpi::PhysicalSize;

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

    let mut app_state = ImageApplication::new(window, &FRAMEWORK);

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
