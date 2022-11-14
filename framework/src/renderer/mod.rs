use cgmath::{vec4, Matrix4, Vector4};

use crate::Camera2d;

pub mod draw_command;
pub mod render_pass_bindable;
pub mod renderer;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RenderCallPerFrameData {
    ortho_matrix: Matrix4<f32>,
    current_time: Vector4<f32>,
}

unsafe impl bytemuck::Pod for RenderCallPerFrameData {}
unsafe impl bytemuck::Zeroable for RenderCallPerFrameData {}

impl RenderCallPerFrameData {
    pub(crate) fn new(camera: &Camera2d, time: f32) -> Self {
        Self {
            ortho_matrix: camera.view_projection(),
            current_time: vec4(time, 0.0, 0.0, 0.0),
        }
    }
}
