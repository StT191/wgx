
// locals
[[block]] struct Matrix { matrix: mat4x4<f32>; };

[[group(0), binding(0)]] var<uniform> clip: Matrix;
[[group(0), binding(1)]] var<uniform> pix: Matrix;


struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0), interpolate(flat)]] color: vec4<f32>;
    [[location(1)]] R: vec2<f32>;
    [[location(2), interpolate(flat)]] dim: vec2<f32>;
    [[location(3), interpolate(flat)]] skew: f32;
};


// fn x_angle(vec:vec2<f32>, len:f32) -> f32 {
//     var angle = acos(vec.x / len); // angle towards x axis
//     if (vec.y < 0.0) { angle = -angle; };
//     return angle;
// }

// fn y_angle(vec:vec2<f32>, len:f32) -> f32 {
//     var angle = acos(vec.y / len); // angle towards y axis
//     if (vec.x > 0.0) { angle = -angle; };
//     return angle;
// }


// fn z_rotation(angle: f32) -> mat4x4<f32> {
//     return mat4x4<f32>(
//         vec4<f32>(cos(angle), sin(angle), 0.0, 0.0),
//         vec4<f32>(-sin(angle), cos(angle), 0.0, 0.0),
//         vec4<f32>(0.0, 0.0, 1.0, 0.0),
//         vec4<f32>(0.0, 0.0, 0.0, 1.0),
//     );
// }
[[stage(vertex)]]
fn vs_main(
    [[location(0)]] N: vec2<f32>, // normalized corner
    [[location(1)]] color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    let P = vec4<f32>(N, 0.0, 1.0);

    out.position = clip.matrix * P;
    out.color = color;

    // let X = (pix.matrix * vec4<f32>(1.0, 0.0, 0.0, 1.0)).xy; // x-axis in pixel-space
    // let x = length(X);

    // var projection = z_rotation(-x_angle(X, length(X))) * pix.matrix; // x-rotation-corrected

    // let d = sqrt(2.0) / 2.0;
    // let p_2 = acos(d);

    let X = (pix.matrix * vec4<f32>(1.0, 0.0, 0.0, 1.0)).xy;
    let Y = (pix.matrix * vec4<f32>(0.0, 1.0, 0.0, 1.0)).xy;

    out.dim = vec2<f32>(length(X), length(Y));

    out.skew = dot(X, Y) / (out.dim.x * out.dim.y);

    out.R = N;

    // let H = (1.0-f)*Y - f*X;
    // let W = (1.0-f)*X + f*Y;

    // projection = z_rotation(-y_angle(H, out.dim.y)) * projection; // x-rotation-corrected

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

    // let a = abs(dot(in.R, in.dim) / length(in.dim)); // skew affection factor

    // let a = abs(Rn.x + Rn.y) / (sqrt(2.0)); // skew affection factor
    // let b = abs(-Rn.x + Rn.y) / (sqrt(2.0)); // skew affection factor
    let len = length(in.dim);
    let dim_q = vec2<f32>(-in.dim.x, in.dim.y);

    let a = abs(dot(No, in.dim) / len); // skew affection factor
    let b = abs(dot(No, dim_q) / len); // skew affection factor

    // let a = abs(in.R.x + in.R.y) / (length(in.R) * sqrt(2.0)); // skew affection factor
    // let b = abs(-in.R.x + in.R.y) / (length(in.R) * sqrt(2.0)); // skew affection factor

    let a_c = pow(a, 0.25);
    let b_c = pow(b, 2.0);

    // let PI_2 = asin(1.0);

    // let f = (1.0 - in.skew*b) / (1.0 - in.skew*a);

    // let f = pow(1.0 - in.skew*b, 2.0) / pow(1.0 - in.skew*a, 2.0);
    // let f = sqrt(1.0 - in.skew*b) / sqrt(1.0 - in.skew*a);

    // let f = (1.0 - pow(in.skew*b, 2.0)) / (1.0 - pow(in.skew*a, 2.0));
    let skew = abs(in.skew);
    let sf = select(1.0, -1.0, in.skew < 0.0);
    let skew_c = pow(skew, 1.0);

    // let f = select(
    //     (1.0 - skew * a) / (1.0 - skew * b),
    //     (1.0 - skew * b) / (1.0 - skew * a),
    //     in.skew > 0.0
    // );

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