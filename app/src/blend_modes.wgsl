fn blend_normal(bottom: vec4<f32>,  top: vec4<f32>) -> vec4<f32> {
    return mix(bottom, top, top.a);
}

fn blend_multiply(bottom: vec4<f32>,  top: vec4<f32>) -> vec4<f32> {
    return bottom * top;
}

fn blend_screen(bottom: vec4<f32>,  top: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(1.0) - ((vec4<f32>(1.0) - top) * (vec4<f32>(1.0) - bottom));
}

let BLEND_NORMAL: i32 = 0;
let BLEND_MULTIPLY: i32 = 1;
let BLEND_SCREEN: i32 = 2;

fn select_blend_mode(mode: i32, bottom: vec4<f32>,  top: vec4<f32>) -> vec4<f32> {
    if (mode == BLEND_NORMAL) {
        return blend_normal(bottom, top);
    } else if (mode == BLEND_MULTIPLY) {
        return blend_multiply(bottom, top);
    } else if (mode == BLEND_SCREEN) {
        return blend_screen(bottom, top);
    }
    
    return vec4<f32>(0.9, 0.0, 0.3, 1.0);    
}