
#include "../imports/uniform_const.wgsl"
#include "../imports/sdf.wgsl"

// ray marching
const START_DIST = 1e-2; // start with a reasonable offset from surface dist // declared externally
const SURFACE_DIST = 1e-4;
const MAX_DEPTH = 3000.0;
const MAX_ITER = 48;

// colors
const bgColor = vec4f(0.01, 0.5, 0.8, 1.0);
const Color = vec4f(1.0, 0.0, 0.0, 1.0);

// sdf map
fn sdMap(P: vec3f, map_color: bool) -> RayField {

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

    return RayField(h, Color);
}


// camera
const cDim = 500.0; // half camera-y dimension
const cDist = 3000.0; // camera field of view distance
const Cp = vec3f(0.0, 0.0, 0.0);
const Cd = vec3f(0.0, -0.5, 2.5);


// lighting
const Ld = vec3f(-0.3, -1.0, -1.5); // light direction
const LL = vec2f(0.02, 0.10); // light levels (min, min lit)

const shDist = 0.0; // max shadow distance

const hL = 0.15; // highlight
const hlPow = 50.0; // highlight power


// import ray marcher
#include "../imports/ray_marcher.wgsl"
