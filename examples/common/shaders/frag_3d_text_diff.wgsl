
@group(0) @binding(0) var<uniform> clip_matrix: mat4x4f;
@group(0) @binding(1) var<uniform> light_matrix: mat4x4f;


struct VertexData {
    @builtin(position) position: vec4f,
    @location(0) tex_coord: vec2f,
    @location(1) lf: f32,
}


@group(0) @binding(2) var color_texture: texture_2d<f32>;
@group(0) @binding(3) var color_sampler: sampler;


override LL_m = 0.01; // light level min
override LL_ml = 0.06; // light level min lit


const hL = 0.15; // highlights
const hlPow = 5.0; // highlight power

fn highlight(Rd: vec3f, N: vec3f, Ln: vec3f) -> f32 {
    let Lr = Ln - 2.0*dot(Ln, N) * N;
    return pow(max(dot(Rd, -Lr), 0.0), hlPow) * hL;
}


@fragment
fn fs_main(in: VertexData) -> @location(0) vec4f {

    let color = textureSample(color_texture, color_sampler, in.tex_coord);

    var lf: f32;

    if (in.lf > 0.0) {
        lf = mix(LL_ml, 1.0, in.lf);
    }
    else {
        lf = mix(LL_m, LL_ml, 1.0 + in.lf);
    }

    return vec4f(color.xyz * lf, 1.0);
}
