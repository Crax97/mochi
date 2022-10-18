let BLEND_NORMAL: i32 = 0;
let BLEND_MULTIPLY: i32 = 1;
let BLEND_SCREEN: i32 = 2;
let BLEND_OVERLAY: i32 = 3;
let BLEND_SOFT_LIGHT: i32 = 4;
let BLEND_COLOR_DODGE: i32 = 5;
let BLEND_COLOR_BURN: i32 = 6;
let BLEND_ADD: i32 = 7;
let BLEND_DIV: i32 = 8;
let BLEND_SUB: i32 = 9;
let BLEND_DIF: i32 = 10;
let BLEND_DARKEN: i32 = 11;
let BLEND_LIGHTEN: i32 = 12;

fn blend_normal(bottom: vec3<f32>,  top: vec3<f32>) -> vec3<f32> {
    return top;
}

fn blend_multiply(bottom: vec3<f32>,  top: vec3<f32>) -> vec3<f32> {
    return bottom * top;
}

fn blend_screen(bottom: vec3<f32>,  top: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(1.0) - ((vec3<f32>(1.0) - bottom) * (vec3<f32>(1.0) - top));
}

fn blend_overlay_f(bottom: f32, top: f32) -> f32 {
    if bottom < 0.5 {
        return 2.0 * bottom * top;
    } else {
        return 1.0 - 2.0 * (1.0 - bottom) * (1.0 - top);
    }
}

fn blend_overlay(bottom: vec3<f32>,  top: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        blend_overlay_f(bottom.r, top.r),
        blend_overlay_f(bottom.g, top.g),
        blend_overlay_f(bottom.b, top.b)
    );
}

fn blend_soft_light(bottom: vec3<f32>,  top: vec3<f32>) -> vec3<f32> {
    return (vec3<f32>(1.0) - 2.0 * top) * bottom * bottom + 2.0 * top * bottom;
}

fn blend_color_dodge(bottom: vec3<f32>,  top: vec3<f32>) -> vec3<f32> {
    return bottom / (1.0 - top);
}

fn blend_color_burn(bottom: vec3<f32>,  top: vec3<f32>) -> vec3<f32> {
    return 1.0 - (bottom / top);
}

fn blend_arith_add(bottom: vec3<f32>,  top: vec3<f32>) -> vec3<f32> {
    return bottom + top;
}

fn blend_arith_div(bottom: vec3<f32>,  top: vec3<f32>) -> vec3<f32> {
    return top / bottom;
}

fn blend_arith_sub(bottom: vec3<f32>,  top: vec3<f32>) -> vec3<f32> {
    return top - bottom;
}

fn blend_arith_difference_f(bottom: f32, top: f32) -> f32 {
    if bottom > top {
        return bottom - top;
    } else {
        return top - bottom;
    }
}

fn blend_arith_difference(bottom: vec3<f32>,  top: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        blend_arith_difference_f(bottom.r, top.r),
        blend_arith_difference_f(bottom.g, top.g),
        blend_arith_difference_f(bottom.b, top.b)
    );
}
fn blend_darken_f(bottom: f32, top: f32) -> f32 {
    return min(bottom, top);
}
fn blend_darken(bottom: vec3<f32>,  top: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(blend_darken_f(bottom.r, top.r), blend_darken_f(bottom.g, top.g), blend_darken_f(bottom.b, top.b));
}
fn blend_lighten_f(bottom: f32, top: f32) -> f32 {
    return max(bottom, top);
}
fn blend_lighten(bottom: vec3<f32>,  top: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(blend_lighten_f(bottom.r, top.r), blend_lighten_f(bottom.g, top.g), blend_lighten_f(bottom.b, top.b));
}

fn select_blend_mode(mode: i32, bottom: vec3<f32>,  top: vec3<f32>) -> vec3<f32> {
    if (mode == BLEND_NORMAL) {
        return blend_normal(bottom, top);
    } else if (mode == BLEND_MULTIPLY) {
        return blend_multiply(bottom, top);
    } else if (mode == BLEND_SCREEN) {
        return blend_screen(bottom, top);
    }else if (mode == BLEND_OVERLAY) {
        return blend_overlay(bottom, top);
    } else if (mode == BLEND_SOFT_LIGHT) {
        return blend_soft_light(bottom, top);
    }else if (mode == BLEND_COLOR_DODGE) {
        return blend_color_dodge(bottom, top);
    } else if (mode == BLEND_COLOR_BURN) {
        return blend_color_burn(bottom, top);
    } else if (mode == BLEND_ADD) {
        return blend_arith_add(bottom, top);
    } else if (mode == BLEND_DIV) {
        return blend_arith_div(bottom, top);
    } else if (mode == BLEND_SUB) {
        return blend_arith_sub(bottom, top);
    } else if (mode == BLEND_DIF) {
        return blend_arith_difference(bottom, top);
    } else if (mode == BLEND_DARKEN) {
        return blend_darken(bottom, top);
    } else if (mode == BLEND_LIGHTEN) {
        return blend_lighten(bottom, top);
    }
    
    return vec3<f32>(0.9, 0.0, 0.3);    
}