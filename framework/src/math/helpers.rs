use std::ops::Index;
use cgmath::{BaseFloat, InnerSpace, Matrix, Matrix3, Matrix4, One, point2, point3, Rad, SquareMatrix, vec2, vec3, Vector3, Vector4, VectorSpace};
use cgmath::num_traits::Float;
use cgmath::num_traits::real::Real;
use crate::Transform2d;

pub fn decompose_no_shear_2d(matrix: Matrix4<f32>) -> Transform2d {
    let translation = matrix.w.truncate();
    let rotation_matrix = matrix.transpose();
    let rotation_matrix = Matrix3 {
        x: rotation_matrix[0].truncate(),
        y: rotation_matrix[1].truncate(),
        z: rotation_matrix[2].truncate(),
    };
    let s_x = rotation_matrix.row(0).magnitude();
    let s_y = rotation_matrix.row(1).magnitude();
    let s_z = rotation_matrix.row(2).magnitude();
    let scale = vec3(s_x, s_y, s_z);
    let mut rotation_matrix = rotation_matrix.transpose();
    rotation_matrix[0] /= s_x;
    rotation_matrix[1] /= s_y;
    rotation_matrix[2] /= s_z;
    rotation_matrix = rotation_matrix.transpose();
    let rot_y_rads = -rotation_matrix.row(0).z.asin();
    let c_b = rot_y_rads.cos();
    let (rot_x_rads, rot_z_rads) = if c_b != 0.0 {
        let rot_x_rads = (rotation_matrix.row(1)[2] / c_b).asin();
        let rot_z_rads = (rotation_matrix.row(0)[0] / c_b).acos();
        (rot_x_rads, rot_z_rads)
    } else {
        let rot_z_rads = 0.0;
        let rot_x_rads = (rotation_matrix.row(1)[0] / c_b).asin();
        (rot_x_rads, rot_z_rads)
    };
    let rotation = vec3(rot_x_rads, rot_y_rads, rot_z_rads);
    Transform2d {
        position: point3(translation.x, translation.y, translation.z),
        scale: vec2(scale.x, scale.y),
        rotation_radians: Rad(rot_z_rads)
    }
}

#[cfg(test)]
mod test {
    use std::f32::consts::PI;
    use cgmath::{SquareMatrix};
    use super::*;

    #[test]
    fn assert_identity() {
        let m1 = Matrix4::identity();
        let decomposed = super::decompose_no_shear_2d(m1);
        assert_eq!(decomposed.position, point3(0.0, 0.0, 0.0));
        assert_eq!(decomposed.scale, vec2(1.0, 1.0));
        assert_eq!(decomposed.rotation_radians, Rad(0.0));
    }
    #[test]
    fn assert_translation() {
        let mut t1 = Transform2d::default();
        t1.translate(vec3(10.0, 10.0, 10.0));
        let decomposed = super::decompose_no_shear_2d(t1.matrix());
        assert_eq!(decomposed.position, point3(10.0, 10.0, 10.0));
        assert_eq!(decomposed.scale, vec2(1.0, 1.0));
        assert_eq!(decomposed.rotation_radians, Rad(0.0));
    }
    #[test]
    fn assert_rotate() {
        let mut t1 = Transform2d::default();
        t1.rotate_radians(PI);
        let decomposed = super::decompose_no_shear_2d(t1.matrix());
        assert_eq!(decomposed.position, point3(0.0, 0.0, 0.0));
        assert_eq!(decomposed.scale, vec2(1.0, 1.0));
        assert_eq!(decomposed.rotation_radians, Rad(PI));
    }
    #[test]
    fn assert_scale() {
        let mut t1 = Transform2d::default();
        t1.scale(vec2(1.5, 1.5));
        let decomposed = super::decompose_no_shear_2d(t1.matrix());
        assert_eq!(decomposed.position, point3(0.0, 0.0, 0.0));
        assert_eq!(decomposed.scale, vec2(2.5, 2.5));
        assert_eq!(decomposed.rotation_radians, Rad(0.0));
    }
    #[test]
    fn assert_full_blown() {
        let mut t1 = Transform2d::default();
        t1.translate(vec3(10.0, 10.0, 10.0));
        t1.rotate_radians(PI);
        t1.scale(vec2(1.5, 1.5));
        let decomposed = super::decompose_no_shear_2d(t1.matrix());
        assert_eq!(decomposed.position, point3(10.0, 10.0, 10.0));
        assert_eq!(decomposed.rotation_radians, Rad(PI));
        assert_eq!(decomposed.scale, vec2(2.5, 2.5));
    }
}