use crate::Transform2d;
use cgmath::{point3, vec2, Matrix, Matrix4, Rad};
use nalgebra::base::Matrix3 as Mat3;

pub fn decompose_no_shear_2d(matrix: Matrix4<f32>) -> Transform2d {
    let matrix = matrix.transpose(); // Since cgmath matrices are column major
    let (sr11, sr12, sr13, t1, sr21, sr22, sr23, t2, sr31, sr32, sr33, t3, _a, _b, _c, _w) = (
        matrix.x[0],
        matrix.x[1],
        matrix.x[2],
        matrix.x[3],
        matrix.y[0],
        matrix.y[1],
        matrix.y[2],
        matrix.y[3],
        matrix.z[0],
        matrix.z[1],
        matrix.z[2],
        matrix.z[3],
        matrix.w[0],
        matrix.w[1],
        matrix.w[2],
        matrix.w[3],
    );

    let translation = point3(t1, t2, t3);

    let sr =
        Mat3::from_iterator([sr11, sr12, sr13, sr21, sr22, sr23, sr31, sr32, sr33].into_iter());
    let (s, r) = sr.polar();
    let rotation = nalgebra::geometry::Rotation3::from_matrix(&r);
    let (_rot_x, _rot_y, rot_z) = rotation.euler_angles();

    Transform2d {
        position: point3(translation.x, translation.y, translation.z),
        scale: vec2(s[0], s[4]),
        rotation_radians: Rad(-rot_z),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cgmath::{vec3, AbsDiffEq, SquareMatrix};
    use std::f32::consts::PI;

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
        assert!(decomposed
            .rotation_radians
            .abs_diff_eq(&Rad(PI / 4.0), 0.005));
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
        t1.rotate_radians(PI / 4.0);
        t1.scale(vec2(1.5, 1.5));
        let decomposed = super::decompose_no_shear_2d(t1.matrix());
        assert!(decomposed
            .position
            .abs_diff_eq(&point3(10.0, 10.0, 10.0), 0.005));
        assert!(decomposed
            .rotation_radians
            .abs_diff_eq(&Rad(PI / 4.0), 0.005));
        assert!(decomposed.scale.abs_diff_eq(&vec2(2.5, 2.5), 0.005));
    }
}
