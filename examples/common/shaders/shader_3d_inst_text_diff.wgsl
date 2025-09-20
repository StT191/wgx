
#include "frag_3d_text_diff.wgsl"

@vertex
fn vs_main(
    @location(0) position: vec3f,
    @location(1) tex_coord: vec3f,
    @location(2) normal: vec3f,
    @location(3) m0: vec4f,
    @location(4) m1: vec4f,
    @location(5) m2: vec4f,
    @location(6) m3: vec4f,
) -> VertexData {
    var out: VertexData;

    let instance_matrix = mat4x4f(m0, m1, m2, m3);

    out.position = clip_matrix * instance_matrix * vec4f(position.xyz, 1.0);
    out.tex_coord = tex_coord.xy;
    out.lf = -normalize(light_matrix * instance_matrix * vec4f(normal.xyz, 1.0)).z;

    // let Ln = (instance_matrix * light_matrix * vec4f(0.0, 0.0, -1.0, 1.0)).xyz;

    // out.lf = diffuse_light(normal, Ln);
    // out.hl = highlight(normalize(out.position.xyz), normal, Ln);

    return out;
}