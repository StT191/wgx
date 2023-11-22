
// &import * from "../imports/uniform_const.wgsl"
// &import * from "../imports/vertex_flat.wgsl"



@fragment
fn fs_main(in: VertexData) -> @location(0) vec4<f32> {

    let P = in.P * vec2<f32>(1.0 * view.z, 1.0);

    let r = length(P);
    let rl = r / sqrt2;

    var fi = acos(P.x / r);
    if (P.y < 0.0) { fi = pi2 - fi; }

    let vr = 2.0 * rl * pi2 + time / 10.0 * pi2;

    let red = 0.9 * pow(sin(vr + fi), 8.0);
    let green = 0.9 * pow(cos(vr - fi), 8.0);
    // let blue = 0.9 * pow(sin(vr - fi + pi), 8.0);


    return vec4<f32>(red, green, 0.0, 1.0);
}
