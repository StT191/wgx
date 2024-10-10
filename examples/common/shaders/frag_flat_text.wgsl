
struct VertexData {
    @builtin(position) position: vec4f,
    @location(0) tex_coord: vec2f,
};


@group(0) @binding(0) var color_texture: texture_2d<f32>;
@group(0) @binding(1) var color_sampler: sampler;


struct FragmetOutput {
    @location(0) first: vec4f,
    @location(1) second: vec4f,
}

@fragment
fn fs_main(in: VertexData) -> FragmetOutput {
    let color = textureSample(color_texture, color_sampler, in.tex_coord);

    return FragmetOutput(color, vec4f(0.0, 1.0, 0.0, 1.0));
}