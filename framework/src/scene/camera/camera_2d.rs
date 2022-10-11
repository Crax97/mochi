use cgmath::{
    point2, point3, vec2, vec3, Matrix4, Point2, SquareMatrix, Transform, Vector2, Vector3,
};

use crate::Transform2d;

#[derive(Clone, Copy)]
pub struct Camera2d {
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

    pub fn unit() -> Self {
        Camera2d::new(-0.1, 1000.0, [-1.0, 1.0, 1.0, -1.0])
    }

    pub fn set_new_bounds(&mut self, lrtb: [f32; 4]) {
        self.left_right_top_bottom = lrtb;
    }

    pub fn translate(&mut self, delta: Vector2<f32>) {
        self.transform.translate(vec3(delta.x, delta.y, 0.0));
    }

    pub fn set_position(&mut self, new_position: Point2<f32>) {
        self.transform.position = point3(new_position.x, new_position.y, 0.0);
    }

    pub fn position(&self) -> Point2<f32> {
        point2(self.transform.position.x, self.transform.position.y)
    }

    pub fn scale(&mut self, delta: f32) {
        self.transform.scale(vec3(delta, delta, 0.0));
    }

    pub fn set_scale(&mut self, new_scale: f32) {
        self.transform.set_scale(vec3(new_scale, new_scale, 0.0));
    }

    pub fn current_scale(&self) -> f32 {
        self.transform.scale.x
    }

    pub fn view(&self) -> Matrix4<f32> {
        Matrix4::from_nonuniform_scale(
            1.0 / self.transform.scale.x,
            1.0 / self.transform.scale.y,
            1.0 / self.transform.scale.z,
        ) * Matrix4::from_angle_z(self.transform.rotation_radians)
            * Matrix4::from_translation(Vector3 {
                x: self.transform.position.x,
                y: self.transform.position.y,
                z: self.transform.position.z,
            })
    }

    pub fn view_no_scale(&self) -> Matrix4<f32> {
        Matrix4::from_angle_z(self.transform.rotation_radians)
            * Matrix4::from_translation(Vector3 {
                x: self.transform.position.x,
                y: self.transform.position.y,
                z: self.transform.position.z,
            })
    }

    pub fn projection(&self) -> Matrix4<f32> {
        let lrtb = &self.left_right_top_bottom;
        cgmath::ortho(lrtb[0], lrtb[1], lrtb[3], lrtb[2], self.near, self.far)
    }
    pub fn view_projection(&self) -> Matrix4<f32> {
        self.projection() * self.view()
    }
    pub fn ndc_into_world(&self, pos: Point2<f32>) -> Point2<f32> {
        let inv_view_camera = self
            .view_projection()
            .invert()
            .expect("Invalid transform matrix!");
        let v4 = inv_view_camera.transform_point(point3(pos.x, pos.y, 0.0));
        point2(v4.x, v4.y)
    }
    pub fn vec_ndc_into_world(&self, pos: Vector2<f32>) -> Vector2<f32> {
        let inv_view_camera = self
            .view_projection()
            .invert()
            .expect("Invalid transform matrix!");
        let v3 = inv_view_camera.transform_vector(vec3(pos.x, pos.y, 0.0));
        vec2(v3.x, v3.y)
    }

    pub fn width(&self) -> f32 {
        (self.left_right_top_bottom[1] - self.left_right_top_bottom[0]).abs()
    }

    pub fn height(&self) -> f32 {
        (self.left_right_top_bottom[2] - self.left_right_top_bottom[3]).abs()
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Camera2dUniformBlock {
    ortho_matrix: Matrix4<f32>,
}

unsafe impl bytemuck::Pod for Camera2dUniformBlock {}
unsafe impl bytemuck::Zeroable for Camera2dUniformBlock {}

impl From<&Camera2d> for Camera2dUniformBlock {
    fn from(camera: &Camera2d) -> Self {
        Self {
            ortho_matrix: camera.view_projection(),
        }
    }
}
