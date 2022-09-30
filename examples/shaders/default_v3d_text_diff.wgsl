
@group(0) @binding(0) var<uniform> clip_matrix: mat4x4<f32>;
@group(0) @binding(1) var<uniform> light_matrix: mat4x4<f32>;


struct VertexData {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) lf: f32,
}


@group(0) @binding(2) var color_texture: texture_2d<f32>;
@group(0) @binding(3) var color_sampler: sampler;


let LL = vec2<f32>(0.02, 0.10); // light levels (min, min lit)


@fragment
fn fs_main(in: VertexData) -> @location(0) vec4<f32> {

    let color = textureSample(color_texture, color_sampler, in.tex_coord);

    var lf: f32;

    if (in.lf > 0.0) {
        lf = mix(LL.y, 1.0, in.lf);
    }
    else {
        lf = mix(LL.x, LL.y, 1.0 + in.lf);
    }

    return vec4<f32>(color.xyz * lf, 1.0);
}
