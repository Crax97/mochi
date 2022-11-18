mod image_editor_app_loop;
mod toolbox;
pub mod tools;
mod ui;

use application::{AppDescription, Application};
use image_editor_app_loop::ImageApplication;
use tools::*;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    Application::<()>::new(AppDescription {
        initial_width: 800,
        initial_height: 600,
    })?
    .run::<String, ImageApplication>()?;
    Ok(())
}
