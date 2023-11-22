
/* &import * from "frag_3d_text_diff.wgsl" */

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec3<f32>,
    @location(2) normal: vec3<f32>,
) -> VertexData {
    var out: VertexData;

    out.position = clip_matrix * vec4<f32>(position.xyz, 1.0);
    out.tex_coord = tex_coord.xy;
    out.lf = -(light_matrix * vec4<f32>(normal.xyz, 1.0)).z;

    return out;
}