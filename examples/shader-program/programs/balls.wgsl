
// &include "../imports/uniform_const.wgsl"
// &include "../imports/sdf.wgsl"

// ray marching
const START_DIST = 1e-2; // start with a reasonable offset from surface dist // declared externally
const SURFACE_DIST = 1e-4;
const MAX_DEPTH = 3000.0;
const MAX_ITER = 64;

// colors
const bgColor = vec4<f32>(0.01, 0.5, 0.8, 1.0);

// sdf map
fn sdMap(P: vec3<f32>, map_color: bool) -> RayField {

    // let s1 = sdSphere(P, vec3<f32>(-400.0 - 175.0*sin(time), 120.0, 400.0 - 175.0*cos(time)), 100.0);
    // let s2 = sdSphere(P, vec3<f32>(0.0, 220.0 + 120.0*cos(time), 500.0), 100.0);
    // let s3 = sdSphere(P, vec3<f32>(400.0, 120.0, 120.0), 100.0 * (0.75 + 0.25*sin(time)));

    let s1 = sdSphere(P, vec3<f32>(-520.0, 50.0*cos(time), 0.0), 200.0);
    let s2 = sdSphere(P, vec3<f32>(0.0, 500.0 + 50.0*sin(time), 0.0), 200.0);
    let s3 = sdBox(P, vec3<f32>(220.0, 50.0*cos(time), 0.0), vec3<f32>(100.0, 100.0, 100.0));

    // let s = opSmoothUnion(s1, opSmoothUnion(s2, s3, 50.0), 50.0);

    let f = P.y + 250.0; // floor


    // returning
    var field = RayField(MAX_DEPTH, bgColor); // color 1

    if (s1 < field.dist) { field.dist = s1; field.color = vec4<f32>(0.0, 1.0, 0.0, 1.0); }
    if (s2 < field.dist) { field.dist = s2; field.color = vec4<f32>(1.0, 0.0, 0.0, 1.0); }
    if (s3 < field.dist) { field.dist = s3; field.color = vec4<f32>(0.4, 0.8, 0.9, 1.0); }

    if (f < field.dist) { field.dist = f; field.color = vec4<f32>(0.8, 0.0, 0.0, 1.0); }

    return field;
}


// camera
const cDim = 500.0; // half camera-y dimension
const cDist = 3000.0; // camera field of view distance
const Cp = vec3<f32>(0.0, 500.0, -1000.0);
const Cd = vec3<f32>(0.0, -0.5, 3.0);


// lighting
const Ld = vec3<f32>(-0.3, -1.0, 0.5); // light direction
const LL = vec2<f32>(0.02, 0.10); // light levels (min, min lit)

const shDist = 1000.0; // max shadow distance

const hL = 0.15; // highlight
const hlPow = 5.0; // highlight power


// import ray marcher
// &include "../imports/ray_marcher.wgsl"
