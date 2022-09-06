use cgmath::{Matrix4, SquareMatrix, Vector3};

use crate::{
    framework::{Framework, TypedBuffer, TypedBufferConfiguration},
    Transform2d,
};

pub(crate) struct Camera2d<'framework> {
    transform: Transform2d,
    pub near: f32,
    pub far: f32,
    pub left_right_top_bottom: [f32; 4],

    camera_buffer: TypedBuffer<'framework>,
}

impl<'framework> Camera2d<'framework> {
    pub fn new(
        near: f32,
        far: f32,
        left_right_top_bottom: [f32; 4],
        framework: &'framework Framework,
    ) -> Self {
        assert!(far > near);

        let camera_buffer =
            framework.allocate_typed_buffer::<Camera2dUniformBlock>(TypedBufferConfiguration {
                initial_data: vec![Camera2dUniformBlock {
                    ortho_matrix: Matrix4::identity(),
                }],
                buffer_type: crate::framework::BufferType::Uniform,
                allow_write: true,
                allow_read: false,
            });

        let mut new_camera = Self {
            transform: Transform2d::default(),
            near,
            far,
            left_right_top_bottom,
            camera_buffer,
        };
        new_camera.update_camera_buffer();
        new_camera
    }

    pub(crate) fn set_new_bounds(&mut self, new_left_right_top_bottom: [f32; 4]) {
        self.left_right_top_bottom = new_left_right_top_bottom;
        self.update_camera_buffer();
    }

    fn update_camera_buffer(&mut self) {
        /*
        self.camera_buffer.write_sync(
            &[Camera2dUniformBlock::from(self as &Camera2d)],
            ,
        )
         */
    }

    pub fn buffer(&self) -> &TypedBuffer {
        &self.camera_buffer
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct Camera2dUniformBlock {
    ortho_matrix: Matrix4<f32>,
}

unsafe impl bytemuck::Pod for Camera2dUniformBlock {}
unsafe impl bytemuck::Zeroable for Camera2dUniformBlock {}

impl From<&Camera2d<'_>> for Camera2dUniformBlock {
    fn from(camera: &Camera2d) -> Self {
        let transform = &camera.transform;
        let lrtb = &camera.left_right_top_bottom;
        let view = Matrix4::from_translation(Vector3 {
            x: -transform.position.x,
            y: -transform.position.y,
            z: transform.position.z,
        }) * Matrix4::from_angle_z(transform.rotation_radians)
            * Matrix4::from_nonuniform_scale(
                transform.scale.x,
                transform.scale.y,
                transform.scale.z,
            );
        let projection = cgmath::ortho(lrtb[0], lrtb[1], lrtb[3], lrtb[2], camera.near, camera.far);

        Self {
            ortho_matrix: projection * view,
        }
    }
}
