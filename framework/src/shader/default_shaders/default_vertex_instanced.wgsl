//@include :common_definitions
//@include :2d_definitions
//@include :2d_transformations

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_uv: vec2<f32>,
}

struct PerInstanceData {
    @location(2) position_and_size: vec4<f32>,
    @location(3) rotation_flip: vec4<f32>,
    @location(4) multiply_color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> uniform_data: PerFrameData;

@vertex
fn vertex(in: VertexInput, instance: PerInstanceData) -> FragmentInput {
    let OPENGL_CORRECT = mat4x4<f32>(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0
    );

    var out : FragmentInput;
    var vp = OPENGL_CORRECT * uniform_data.vp;
    var trans = translation(instance.position_and_size.xy);
    var rot = rot_z(instance.rotation_flip.x);
    let flip = instance.rotation_flip.y;
    
    var scale = scale(instance.position_and_size.zw);
    var model = rot * scale * trans;
    var projected = vec4<f32>(in.position, 1.0) * model;
    let y = flip * (1.0 - in.tex_uv.y) + (1.0 - flip) * in.tex_uv.y;
    out.coordinates_position = vp * projected;
    out.position = in.position;
    out.scale = scale;
    out.tex_uv = vec2<f32>(in.tex_uv.x, y);
    out.multiply_color = instance.multiply_color;
    return out;
}