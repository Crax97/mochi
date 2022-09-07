use cgmath::{vec3, Matrix4, SquareMatrix, Vector2, Vector3};

use super::super::transform::Transform2d;
use framework::{
    typed_buffer::{BufferType, TypedBuffer, TypedBufferConfiguration},
    Framework,
};

pub struct Camera2d<'framework> {
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
                buffer_type: BufferType::Uniform,
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

    pub fn set_new_bounds(&mut self, new_left_right_top_bottom: [f32; 4]) {
        self.left_right_top_bottom = new_left_right_top_bottom;
        self.update_camera_buffer();
    }

    fn update_camera_buffer(&mut self) {
        self.camera_buffer
            .write_sync(&[Camera2dUniformBlock::from(self as &Camera2d)]);
    }

    pub fn buffer(&self) -> &TypedBuffer {
        &self.camera_buffer
    }

    pub fn translate(&mut self, delta: Vector2<f32>) {
        self.transform.translate(vec3(delta.x, delta.y, 0.0));
        self.update_camera_buffer();
    }

    pub fn view(&self) -> Matrix4<f32> {
        Matrix4::from_translation(Vector3 {
            x: self.transform.position.x,
            y: self.transform.position.y,
            z: self.transform.position.z,
        }) * Matrix4::from_angle_z(self.transform.rotation_radians)
            * Matrix4::from_nonuniform_scale(
                self.transform.scale.x,
                self.transform.scale.y,
                self.transform.scale.z,
            )
    }

    pub fn projection(&self) -> Matrix4<f32> {
        let lrtb = &self.left_right_top_bottom;
        cgmath::ortho(lrtb[0], lrtb[1], lrtb[3], lrtb[2], self.near, self.far)
    }
    pub fn view_projection(&self) -> Matrix4<f32> {
        self.projection() * self.view()
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Camera2dUniformBlock {
    ortho_matrix: Matrix4<f32>,
}

unsafe impl bytemuck::Pod for Camera2dUniformBlock {}
unsafe impl bytemuck::Zeroable for Camera2dUniformBlock {}

impl From<&Camera2d<'_>> for Camera2dUniformBlock {
    fn from(camera: &Camera2d) -> Self {
        Self {
            ortho_matrix: camera.view_projection(),
        }
    }
}
