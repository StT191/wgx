
struct Matrix { matrix: mat4x4<f32>; };

[[group(0), binding(0)]] var<uniform> projection: Matrix;


struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] tex_coord: vec2<f32>;
};


[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] tex_coord: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    out.position = projection.matrix * vec4<f32>(position, 1.0);
    out.tex_coord = tex_coord;

    return out;
}


[[group(0), binding(1)]] var color_texture: texture_2d<f32>;
[[group(0), binding(2)]] var color_sampler: sampler;


[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {

    return textureSample(color_texture, color_sampler, in.tex_coord);
}