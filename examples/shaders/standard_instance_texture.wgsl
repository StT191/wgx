
// &import * from "simple_texture_diffuse.wgsl"

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) m0: vec4<f32>,
    @location(4) m1: vec4<f32>,
    @location(5) m2: vec4<f32>,
    @location(6) m3: vec4<f32>,
) -> VertexData {
    var out: VertexData;

    let inst_matrix = mat4x4<f32>(m0, m1, m2, m3);

    out.position = clip_matrix * inst_matrix * vec4<f32>(position, 1.0);
    out.tex_coord = tex_coord.xy;
    // out.lf = abs(-(light.m * vec4<f32>(normal, 1.0)).z);
    out.lf = -(light_matrix * inst_matrix * vec4<f32>(normal, 1.0)).z;

    return out;
}