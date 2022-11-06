struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_uv: vec2<f32>,
}

struct FragmentInput {
    @builtin(position) coordinates_position: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) scale: vec3<f32>,
    @location(2) tex_uv: vec2<f32>,
    @location(3) multiply_color: vec4<f32>,
}

struct PerFrameData {
    vp: mat4x4<f32>,
}