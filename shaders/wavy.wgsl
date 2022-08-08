
// locals
struct Float { f: f32 };
struct Vec2 { v: vec2<f32> };

@group(0) @binding(0) var<uniform> viewport: Vec2;
@group(0) @binding(1) var<uniform> scale: Vec2;
// @group(0), binding(2) var<uniform> t: Float;

var<push_constant> t: Float;


struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(perspective) R: vec2<f32>,
};


@vertex
fn vs_main(
    @location(0) R: vec2<f32>,
) -> VertexOutput {
    return VertexOutput(vec4<f32>(R, 0.0, 1.0), R);
}

// let pi0 = 1.5707963267948966;
// let pi = 3.141592653589793;
let pi2 = 6.283185307179586;
let sqrt2 = 1.4142135623730951;


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {


    let r = length(in.R);
    let rl = r / sqrt2;

    var fi = acos(in.R.x / r);
    if (in.R.y < 0.0) { fi = pi2 - fi; }


    let vr = 2.0 * rl * pi2 + t.f / 10.0 * pi2;

    let red = 0.9 * pow(sin(vr + fi), 8.0);
    let green = 0.9 * pow(cos(vr - fi), 8.0);
    // let blue = 0.9 * pow(sin(vr - fi + pi), 8.0);


    return vec4<f32>(red, green, 0.0, 1.0);
}





