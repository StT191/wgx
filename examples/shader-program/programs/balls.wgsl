
// &import * from "../imports/uniform_const.wgsl"
// &import * from "../imports/sdf.wgsl"

// ray marching
let MAX_DEPTH = 3000.0;
let MAX_ITER = 64;


// sdf map
fn sdMap(P: vec3<f32>) -> f32 {

    // let s1 = sdSphere(P, vec3<f32>(-400.0 - 175.0*sin(time), 120.0, 400.0 - 175.0*cos(time)), 100.0);
    // let s2 = sdSphere(P, vec3<f32>(0.0, 220.0 + 120.0*cos(time), 500.0), 100.0);
    // let s3 = sdSphere(P, vec3<f32>(400.0, 120.0, 120.0), 100.0 * (0.75 + 0.25*sin(time)));

    let s1 = sdSphere(P, vec3<f32>(-520.0, 50.0*cos(time), 0.0), 200.0);
    let s2 = sdSphere(P, vec3<f32>(0.0, 500.0 + 50.0*sin(time), 0.0), 200.0);
    let s3 = sdBox(P, vec3<f32>(220.0, 50.0*cos(time), 0.0), vec3<f32>(100.0, 100.0, 100.0));

    let s = min(s1, min(s2, s3));
    // let s = opSmoothUnion(s1, opSmoothUnion(s2, s3, 50.0), 50.0);

    let f = P.y + 250.0; // floor

    // return s;
    return min(s, f);
}


// camera
let cDim = 500.0; // half camera-y dimension
let cDist = 3000.0; // camera field of view distance
let Cp = vec3<f32>(0.0, 500.0, -1000.0);
let Cd = vec3<f32>(0.0, -0.5, 3.0);


// lighting
let Ld = vec3<f32>(-0.3, -1.0, 0.5); // light direction
let LL = vec2<f32>(0.02, 0.10); // light levels (min, min lit)

let shDist = 1000.0; // max shadow distance

let hL = 0.15; // highlight
let hlPow = 5.0; // highlight power


// coloring
let Color = vec3<f32>(1.0, 0.0, 0.0);
let bgColor = vec4<f32>(0.01, 0.5, 0.8, 1.0);


// import ray marcher
// &import * from "../imports/ray_marcher.wgsl"
