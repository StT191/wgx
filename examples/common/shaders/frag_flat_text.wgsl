
struct VertexData {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};


@group(0) @binding(0) var color_texture: texture_2d<f32>;
@group(0) @binding(1) var color_sampler: sampler;


struct FragmetOutput {
    @location(0) first: vec4<f32>,
    // @location(1) second: vec4<f32>,
}

@fragment
fn fs_main(in: VertexData) -> FragmetOutput {
    let color = textureSample(color_texture, color_sampler, in.tex_coord);

    return FragmetOutput(color);
}