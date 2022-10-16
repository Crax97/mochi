fn blend_normal(bottom: vec4<f32>,  top: vec4<f32>) -> vec4<f32> {
    // Over operator
    let a = top.a + bottom.a * (1.0 - top.a);
    return (bottom * bottom.a + top * top.a) / a;
}

let BLEND_NORMAL: i32 = 0;

fn blend_mode(mode: i32, bottom: vec4<f32>,  top: vec4<f32>) -> vec4<f32> {
    if (mode == BLEND_NORMAL) {
        return blend_normal(bottom, top);
    }
    
    return vec4<f32>(0.9, 0.0, 0.3, 1.0);    
}