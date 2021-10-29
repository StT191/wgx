
// locals
[[block]] struct Matrix { matrix: mat4x4<f32>; };

[[group(0), binding(0)]] var<uniform> clip: Matrix;
[[group(0), binding(1)]] var<uniform> pix: Matrix;


struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0), interpolate(flat)]] color: vec4<f32>;
    [[location(1), interpolate(perspective)]] E: vec2<f32>;
    [[location(2), interpolate(flat)]] pr0: vec4<f32>;
    [[location(3), interpolate(flat)]] pr1: vec4<f32>;
    [[location(4), interpolate(flat)]] pr2: vec4<f32>;
    [[location(5), interpolate(flat)]] pr3: vec4<f32>;
};


fn translation(x:f32, y:f32, z:f32) -> mat4x4<f32> {
    return mat4x4<f32>(vec4<f32>(1.0, 0.0, 0.0, x), vec4<f32>(0.0, 1.0, 0.0, y), vec4<f32>(0.0, 0.0, 1.0, z), vec4<f32>(0.0, 0.0, 0.0, 1.0));
}

fn normal_2d(vec:vec2<f32>) -> vec2<f32> {
    return vec2<f32>(vec.y, -vec.x);
}

fn homogen_2d(vec:vec4<f32>) -> vec2<f32> {
    return vec2<f32>(vec.x/vec.w, vec.y/vec.w);
}


[[stage(vertex)]]
fn vs_main(
    [[location(0)]] E: vec2<f32>, // normalized corner
    [[location(1)]] color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    out.position = clip.matrix * vec4<f32>(E, 0.0, 1.0);
    out.color = color;
    out.E = E;

    let O = (pix.matrix * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xy; // origin

    let projection = translation(-O.x, -O.y, 0.0) * pix.matrix;

    out.pr0 = projection[0];
    out.pr1 = projection[1];
    out.pr2 = projection[2];
    out.pr3 = projection[3];

    return out;
}


[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {


    let e = length(in.E);

    if (e > 1.0) {
        discard;
        // return vec4<f32>(1.0, 1.0, 0.0, 1.0);
    }


    // get pixel distance to rim
    let prj = mat4x4<f32>(in.pr0, in.pr1, in.pr2, in.pr3);

    let R = homogen_2d(prj * vec4<f32>(in.E, 0.0, 1.0)); // R in pixel space
    // let r = length(R); // length of R in pixel space

    // R at rim
    var Ep = in.E * 1.0/e; // R at rim in unit circle
    var Rp = homogen_2d(prj * vec4<f32>(Ep, 0.0, 1.0)); // R at rim in pixel space
    var rp = length(Rp);
    var dr = distance(R, Rp); // radial distance to rim

    // { // look if opposite side is closer (in case of perspective skewing)

    //     let Rp0 = homogen_2d(prj * vec4<f32>(-Ep, 0.0, 1.0)); // R at rim in pixel space
    //     let rp0 = length(Rp0);
    //     let dr0 = distance(R, Rp0);

    //     if (dr0 < dr) { Ep = -Ep; Rp = Rp0; rp = rp0; dr = dr0; }
    // }

    var ds = dr;

    // for(var i:i32 = 0; i < 10; i = i + 1) {

        let Et = normal_2d(Ep); // tangent-vector

        let T = Rp - homogen_2d(prj * vec4<f32>(Ep + Et, 0.0, 1.0)); // tangent in pixel space
        let t = length(T);

        let cos_fi = dot(Rp, T) / (rp*t);

        // reassign
        // Ep = normalize(Ep + Et * cos_fi * dr/t ); // new E vector
        // Rp = homogen_2d(prj * vec4<f32>(Ep, 0.0, 1.0)); // new Rp vector

        // rp = length(Rp);
        // dr = min(distance(R, Rp), dr * sin(acos(cos_fi)));
        // dr = distance(R, Rp);

        ds = dr * sin(acos(cos_fi));

        // if (dr < ds) { ds = dr; }
    // }

    var color = in.color;

    let g = 1.0 * sqrt(2.0); // fade threshold

    if (ds < g) {
        color.a = color.a * (ds / g);
        // color.g = 1.0 - (ds / g);
        // color.g = (ds / g);
        // return vec4<f32>(0.0, 0.0, 1.0, 1.0);
    }

    return color;
}