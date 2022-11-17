use std::f32::consts::PI;

use cgmath::{point3, Matrix4, Point3, Rad, Vector2, Vector3};

#[derive(Clone, Copy)]
pub struct Transform2d {
    pub position: Point3<f32>,
    pub scale: Vector2<f32>,
    pub rotation_radians: Rad<f32>,
}

impl Default for Transform2d {
    fn default() -> Self {
        Self {
            position: point3(0.0, 0.0, 0.0),
            scale: Vector2 { x: 1.0, y: 1.0 },
            rotation_radians: Rad(0.0),
        }
    }
}

impl Transform2d {
    pub fn translate(&mut self, delta: Vector3<f32>) {
        self.position += delta;
    }
    pub fn rotate_degrees(&mut self, degrees: f32) {
        self.rotation_radians += Rad(PI / 180.0 * degrees);
    }
    pub fn rotate_radians(&mut self, radians: f32) {
        self.rotation_radians += Rad(radians);
    }
    pub fn scale(&mut self, delta: Vector2<f32>) {
        self.scale += delta;
    }

    pub(crate) fn set_scale(&mut self, new_scale: Vector2<f32>) {
        self.scale = new_scale;
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(Vector3 {
            x: self.position.x,
            y: self.position.y,
            z: self.position.z,
        }) * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, 1.0)
            * Matrix4::from_angle_z(self.rotation_radians)
    }
}
