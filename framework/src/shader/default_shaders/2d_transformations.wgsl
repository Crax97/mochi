fn rot_z(angle_rads: f32) -> mat4x4<f32> {
    let ca = cos(angle_rads);
    let sa = sin(angle_rads);
    return mat4x4(ca , -sa, 0.0, 0.0,
                  sa , ca , 0.0, 0.0,
                  0.0, 0.0, 1.0, 0.0,
                  0.0, 0.0, 0.0, 1.0);
}

fn scale(s: vec2<f32>) -> mat4x4<f32> {
    return mat4x4(s.x, 0.0, 0.0, 0.0,
                  0.0, s.y, 0.0, 0.0,
                  0.0, 0.0, 1.0, 0.0,
                  0.0, 0.0, 0.0, 1.0);
}

fn translation(pos: vec2<f32>) -> mat4x4<f32> {
    return mat4x4(1.0, 0.0, 0.0, pos.x,
                  0.0, 1.0, 0.0, pos.y,
                  0.0, 0.0, 1.0, 0.0,
                  0.0, 0.0, 0.0, 1.0);
}