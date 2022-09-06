use std::f32::consts::PI;

use cgmath::{point3, Point3, Rad, Vector3};

pub(crate) struct Transform2d {
    pub position: Point3<f32>,
    pub scale: Vector3<f32>,
    pub rotation_radians: Rad<f32>,
}

impl Default for Transform2d {
    fn default() -> Self {
        Self {
            position: point3(0.0, 0.0, 0.0),
            scale: Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            rotation_radians: Rad(0.0),
        }
    }
}

impl Transform2d {
    fn translate(&mut self, delta: Vector3<f32>) {
        self.position += delta;
    }
    fn rotate_degrees(&mut self, degrees: f32) {
        self.rotation_radians += Rad(PI / 180.0 * degrees);
    }
    fn rotate_radians(&mut self, radians: f32) {
        self.rotation_radians += Rad(radians);
    }
    fn scale(&mut self, delta: Vector3<f32>) {
        self.scale += delta;
    }
}
