use cgmath::{Matrix4, Vector3};

use crate::Transform2d;

pub(crate) struct Camera2d {
    transform: Transform2d,
    pub near: f32,
    pub far: f32,
    pub left_right_top_bottom: [f32; 4],
}

impl Camera2d {
    pub fn new(near: f32, far: f32, left_right_top_bottom: [f32; 4]) -> Self {
        assert!(far > near);
        Self {
            transform: Transform2d::default(),
            near,
            far,
            left_right_top_bottom,
        }
    }
}

pub(crate) struct Camera2dUniformBlock {
    ortho_matrix: Matrix4<f32>,
}

impl From<&Camera2d> for Camera2dUniformBlock {
    fn from(camera: &Camera2d) -> Self {
        let transform = &camera.transform;
        let lrtb = &camera.left_right_top_bottom;
        let view = Matrix4::from_translation(Vector3 {
            x: transform.position.x,
            y: transform.position.y,
            z: transform.position.z,
        }) * Matrix4::from_angle_z(transform.rotation_radians)
            * Matrix4::from_nonuniform_scale(
                transform.scale.x,
                transform.scale.y,
                transform.scale.z,
            );
        let projection = cgmath::ortho(lrtb[0], lrtb[1], lrtb[3], lrtb[2], camera.near, camera.far);

        Self {
            ortho_matrix: view * projection,
        }
    }
}
