
// &import * from "../imports/uniform_const.wgsl"
// &import * from "../imports/vertex_flat.wgsl"



fn reduce(fi: f32, p: f32) -> f32 {
    return pow(abs(cos(fi)), p);
}

fn s_odd(val: f32) -> f32 {
    if (val > 0.0) { return floor(val % 2.0); }
    else { return ceil((val - 1.0) % 2.0); }
}


fn stx(Pa: vec2<f32>) -> f32 {
    let S = (fract(Pa) - 0.5) * 2.0;
    let D = (abs(vec2<f32>(s_odd(Pa.x), s_odd(Pa.y))) - 0.5) * 2.0;

    let S = S + 0.2 * D * vec2<f32>(cos(time), sin(time));
    let S = S + 0.2 * vec2<f32>(sin(Pa.y*pi1_2 + time), sin(Pa.x*pi1_2 + time));

    return smoothstep(0.96, 1.0, 1.0 - length(S));
}

let amt_x = 15.0;
let amt_y = 25.0;
let amt_h = 35.0;

let white = vec3<f32>(1.0, 1.0, 1.0);
let frame_cl = vec3<f32>(0.04, 0.0, 0.0);

fn sin_ab(a: f32, t: f32, b: f32) -> f32 {
    return sin(a*t*pi1_2 + b);
}

fn cos_ab(a: f32, t: f32, b: f32) -> f32 {
    return cos(a*t*pi1_2 + b);
}

fn st(a: f32, t: f32, b: f32, p: f32) -> f32 {
    return pow(1.0 - abs(cos_ab(a, t, b)), p);
}


@fragment
fn fs_main(in: VertexData) -> @location(0) vec4<f32> {

    let P = in.P * vec2<f32>(1.0, 1.0/viewport.z);

    let f = 0.0
        + st(20.0, P.x, 0.9 * sin_ab(5.0, P.x, 0.0), 8.0)
            * st(10.0, P.y, 0.0, 8.0)
    ;

    // let Px = in.P * vec2<f32>(1.0, 1.0/viewport.z);
    // let Pax = amt_x/2.0 * Px;

    // let Py = in.P * vec2<f32>(1.0*viewport.z, 1.0);
    // let Pay = amt_y/2.0 * Py;

    // let Pah = amt_h/2.0 * Py;

    // let f = stx(Pax) + stx(Pay) + stx(Pah);

    return vec4<f32>(f * white, 1.0);
}
