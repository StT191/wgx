
@group(0) @binding(0) var<uniform> projection: mat4x4f;

/* &include "frag_flat_text.wgsl" */

@vertex
fn vs_main(
    @location(0) position: vec3f,
    @location(1) tex_coord: vec2f,
) -> VertexData {
    var out: VertexData;

    out.position = projection * vec4f(position, 1.0);
    out.tex_coord = tex_coord;

    return out;
}