
/* &include "frag_3d_text_diff.wgsl" */

@vertex
fn vs_main(
    @location(0) position: vec3f,
    @location(1) tex_coord: vec3f,
    @location(2) normal: vec3f,
) -> VertexData {
    var out: VertexData;

    out.position = clip_matrix * vec4f(position.xyz, 1.0);
    out.tex_coord = tex_coord.xy;
    out.lf = -(light_matrix * vec4f(normal.xyz, 1.0)).z;

    return out;
}