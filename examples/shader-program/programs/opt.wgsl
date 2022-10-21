
// &import * from "../imports/uniform_const.wgsl"
// &import * from "../imports/sdf.wgsl"

// ray marching
let MAX_DEPTH = 3000.0;
let MAX_ITER = 48;


// sdf map
fn sdMap(P: vec3<f32>) -> f32 {

    let h =
        P.y +
        0.5 * max(1.0 - 1.01 * P.z/MAX_DEPTH, 0.0) *
        (
            (
                2.5*sin(P.z/8.0 + 10.0*time) * 2.0*cos(P.x/5.0 + 13.0*time)
            ) + (
                9.0*sin(P.z/280.0 - time - 0.1 * cos(P.x / 60.0)) +
                4.0*sin(P.z/150.0 + 0.0)
            ) * (
                11.0*cos(P.x/310.0 - pi + time + 0.1 * cos(P.z / 50.0)) +
                5.0*cos(P.x/120.0 - 0.0)
            )
        )
    ; // floor

    return h;
}


// camera
let cDim = 500.0; // half camera-y dimension
let cDist = 3000.0; // camera field of view distance
let Cp = vec3<f32>(0.0, 0.0, 0.0);
let Cd = vec3<f32>(0.0, -0.5, 2.5);


// lighting
let Ld = vec3<f32>(-0.3, -1.0, -1.5); // light direction
let LL = vec2<f32>(0.02, 0.10); // light levels (min, min lit)

let shDist = 0.0; // max shadow distance

let hL = 0.15; // highlight
let hlPow = 50.0; // highlight power


// coloring
let Color = vec3<f32>(1.0, 0.0, 0.0);
let bgColor = vec4<f32>(0.01, 0.5, 0.8, 1.0);


// import ray marcher
// &import * from "../imports/ray_marcher.wgsl"
