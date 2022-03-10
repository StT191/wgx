
// locals
struct Matrix { matrix: mat4x4<f32>; };

[[group(0), binding(0)]] var<uniform> clip: Matrix;
[[group(0), binding(1)]] var<uniform> pix: Matrix;


struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0), interpolate(flat)]] color: vec4<f32>;
    [[location(1)]] R: vec2<f32>;
    [[location(2), interpolate(flat)]] dim: vec2<f32>;
    [[location(3), interpolate(flat)]] skew: f32;
};


[[stage(vertex)]]
fn vs_main(
    [[location(0)]] N: vec2<f32>, // normalized corner
    [[location(1)]] color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    let P = vec4<f32>(N, 0.0, 1.0);

    out.position = clip.matrix * P;
    out.color = color;

    let X = (pix.matrix * vec4<f32>(1.0, 0.0, 0.0, 1.0)).xy;
    let Y = (pix.matrix * vec4<f32>(0.0, 1.0, 0.0, 1.0)).xy;

    out.dim = vec2<f32>(length(X), length(Y));

    out.skew = dot(X, Y) / (out.dim.x * out.dim.y);

    out.R = N;

    return out;
}


[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {

    let R = in.R * in.dim;
    let r = length(R);
    let Rn = R / r; // = (cos_a, sin_a)

    let re = in.dim.x * in.dim.y / length(Rn * vec2<f32>(in.dim.y, in.dim.x)); // distance at edge


    let N = re*Rn / (in.dim * in.dim);
    let No = N/length(N); // normal

    let len = length(in.dim);
    let dim_q = vec2<f32>(-in.dim.x, in.dim.y);

    let a = abs(dot(No, in.dim) / len); // skew affection factor
    let b = abs(dot(No, dim_q) / len); // skew affection factor

    let a_c = pow(a, 0.25);
    let b_c = pow(b, 2.0);

    let skew = abs(in.skew);
    let sf = select(1.0, -1.0, in.skew < 0.0);
    let skew_c = pow(skew, 1.0);


    let m = 0.5;
    let n = sqrt(0.5);

    let f = 1.0 + n*a_c * sf*skew_c - m*b_c * sf*skew_c;

    if (r > re) {
        return vec4<f32>(1.0, 1.0, 0.0, 1.0); // yellow
        // discard;
    }

    let ds = (re - r) * dot(Rn, No) * f; // delta to edge in pixel space, skew-corrected

    var color = in.color;

    if (a > 0.995 || b > 0.995) {
        color.b = 1.0;
    }


    let g = 50.0; // blur threshold

    if (ds < g) {
        // color.a = color.a * (ds / g));
        color.g = (ds / g);
    }

    return color;
}