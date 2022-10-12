struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_uv: vec2<f32>,
}

struct PerInstanceData {
    position_and_size: vec4<f32>,
    rotation_flip_opacity: vec4<f32>,
}

struct PerFrameData {
    vp: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniform_data: PerFrameData;

@group(1) @binding(0)
var<uniform> instance_data: PerInstanceData;

struct VertexOutput {
    @builtin(position) coordinates_position: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) tex_uv: vec2<f32>,
    @location(2) opacity: f32,
}

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

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    let OPENGL_CORRECT = mat4x4<f32>(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0
    );

    var out : VertexOutput;
    var trans = translation(instance_data.position_and_size.xy);
    var rot = rot_z(instance_data.rotation_flip_opacity.x);
    var scale = scale(instance_data.position_and_size.zw);
    var model = rot * scale * trans;
    var projected = vec4<f32>(in.position, 1.0) * model;
    var vp = OPENGL_CORRECT * uniform_data.vp;
    let y = 1.0 - in.tex_uv.y;
    out.coordinates_position = vp * projected;
    out.position = in.position;
    out.tex_uv = vec2<f32>(in.tex_uv.x, y);
    out.opacity = 1.0;
    return out;
}