use std::ops::{Index, MulAssign};
use cgmath::{BaseFloat, InnerSpace, Matrix, Matrix3, Matrix4, One, point2, point3, Rad, SquareMatrix, vec2, vec3, Vector3, Vector4, VectorSpace};
use cgmath::num_traits::Float;
use cgmath::num_traits::real::Real;
use nalgebra::base::Matrix3 as Mat3;
use crate::Transform2d;

pub fn decompose_no_shear_2d(matrix: Matrix4<f32>) -> Transform2d {
    let matrix = matrix.transpose(); // Since cgmath matrices are column major
    let (sr11, sr12, sr13, t1,
         sr21, sr22, sr23, t2,
        sr31, sr32, sr33, t3,
        a, b, c, w) = (matrix.x[0], matrix.x[1], matrix.x[2], matrix.x[3],
                       matrix.y[0], matrix.y[1], matrix.y[2], matrix.y[3],
                       matrix.z[0], matrix.z[1], matrix.z[2], matrix.z[3],
                       matrix.w[0], matrix.w[1], matrix.w[2], matrix.w[3]);
    
    
    let translation = point3(t1, t2, t3);
    
    let SR = Mat3::from_iterator([sr11, sr12, sr13, sr21, sr22, sr23, sr31, sr32, sr33 ].into_iter());
    let (s, r) = SR.polar();
    let rotation = nalgebra::geometry::Rotation3::from_matrix(&r);
    let (rot_x, rot_y, rot_z) = rotation.euler_angles();
    
    Transform2d {
        position: point3(translation.x, translation.y, translation.z),
        scale: vec2(s[0], s[4]),
        rotation_radians: Rad(-rot_z)
    }
}

#[cfg(test)]
mod test {
    use std::f32::consts::PI;
    use cgmath::{AbsDiffEq, SquareMatrix};
    use super::*;

    // Other tests cannot be done, since it's not guaranteed that decomposing a transformation matrix
    // gives back the exact same TRS matrices that were used originally.
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
    fn assert_rotate1() {
        let mut t1 = Transform2d::default();
        t1.rotate_radians(PI / 4.0);
        let decomposed = super::decompose_no_shear_2d(t1.matrix());
        assert_eq!(decomposed.position, point3(0.0, 0.0, 0.0));
        assert!(decomposed.scale.abs_diff_eq(&vec2(1.0, 1.0), 0.005));
        assert!(decomposed.rotation_radians.abs_diff_eq(&Rad(PI / 4.0), 0.005));
    }
    #[test]
    fn assert_scale() {
        let mut t1 = Transform2d::default();
        t1.scale(vec2(1.5, 1.5));
        let decomposed = super::decompose_no_shear_2d(t1.matrix());
        assert_eq!(decomposed.position, point3(0.0, 0.0, 0.0));
        assert!(decomposed.scale.abs_diff_eq(&vec2(2.5, 2.5), 0.005));
        assert_eq!(decomposed.rotation_radians, Rad(0.0));
    }
    #[test]
    fn assert_full_blown() {
        let mut t1 = Transform2d::default();
        t1.translate(vec3(10.0, 10.0, 10.0));
        t1.rotate_radians(PI/4.0);
        t1.scale(vec2(1.5, 1.5));
        let decomposed = super::decompose_no_shear_2d(t1.matrix());
        assert!(decomposed.position.abs_diff_eq(&point3(10.0, 10.0, 10.0), 0.005));
        assert!(decomposed.rotation_radians.abs_diff_eq(&Rad(PI/4.0), 0.005));
        assert!(decomposed.scale.abs_diff_eq(&vec2(2.5, 2.5), 0.005));
    }
}