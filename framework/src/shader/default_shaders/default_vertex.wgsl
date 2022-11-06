//@include :common_definitions
//@include :2d_definitions
//@include :2d_transformations

@group(0) @binding(0)
var<uniform> uniform_data: PerFrameData;

@group(1) @binding(0)
var<uniform> instance_data: PerInstanceData;

@vertex
fn vertex(in: VertexInput) -> FragmentInput {
    let OPENGL_CORRECT = mat4x4<f32>(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0
    );

    var out : FragmentInput;
    var trans = translation(instance_data.position_and_size.xy);
    var rot = rot_z(instance_data.rotation_flip.x);
    let flip_uv_y = instance_data.rotation_flip.y;
    
    var scale = scale(instance_data.position_and_size.zw);
    var model = rot * scale * trans;
    var projected = vec4<f32>(in.position, 1.0) * model;
    var vp = OPENGL_CORRECT * uniform_data.vp;
    let y = (1.0 - in.tex_uv.y) * flip_uv_y + (1.0 - flip_uv_y) * in.tex_uv.y;
    out.coordinates_position = vp * projected;
    out.position = in.position;
    out.scale = vec3<f32>(instance_data.position_and_size.z, instance_data.position_and_size.w, 0.0);
    out.tex_uv = vec2<f32>(in.tex_uv.x, y);
    out.multiply_color = instance_data.multiply_color;
    return out;
}