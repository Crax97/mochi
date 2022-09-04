struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_uv: vec2<f32>,
}

struct PerInstanceData {
    @location(2) position_and_size: vec4<f32>,
    @location(3) rotation_radians: f32,
}


struct VertexOutput {
    @builtin(position) coordinates_position: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) tex_uv: vec2<f32>,
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
    return mat4x4(0.0, 0.0, 0.0, pos.x,
                  0.0, 0.0, 0.0, pos.y,
                  0.0, 0.0, 0.0, 0.0,
                  0.0, 0.0, 0.0, 1.0);
}

@vertex
fn vs(in: VertexInput, instance: PerInstanceData) -> VertexOutput {
    var out : VertexOutput;
    var model = scale(instance.position_and_size.zw) * rot_z(instance.rotation_radians) * translation(instance.position_and_size.xy);
    out.coordinates_position = vec4<f32>(in.position + vec3<f32>(instance.position_and_size.xy, 0.0), 1.0);
    out.position = in.position;
    out.tex_uv = in.tex_uv;
    return out;
}


@group(0) @binding(0) var diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;
@fragment
fn fs(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(diffuse, s_diffuse, in.tex_uv);
}