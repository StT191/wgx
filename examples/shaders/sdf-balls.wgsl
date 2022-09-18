
struct VertexData {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(linear) p: vec2<f32>,
};

@vertex
fn vs_main(@location(0) p: vec2<f32>) -> VertexData {
    return VertexData(vec4<f32>(p, 0.0, 1.0), p);
}

// main

// math constants

// let pi0 = 1.5707963267948966;
// let pi = 3.141592653589793;
// let pi2 = 6.283185307179586;
// let sqrt2 = 1.4142135623730951;


// sdf definitions
fn sdSphere(P: vec3<f32>, pos: vec3<f32>, r: f32) -> f32 {
  return length(P - pos) - r;
}

// operations

fn opSmoothUnion(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (d2-d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) - k*h * (1.0-h);
}

// uniform
@group(0) @binding(0) var<uniform> viewport: vec3<f32>;
@group(0) @binding(1) var<uniform> scale: f32;
// @group(0) @binding(2) var<uniform> time: f32;

var<push_constant> time: f32; // time in secs


// sdf map

fn sdMap(P: vec3<f32>) -> f32 {

    // let s1 = sdSphere(P, vec3<f32>(-400.0 - 175.0*sin(time), 120.0, 400.0 - 175.0*cos(time)), 100.0);
    // let s2 = sdSphere(P, vec3<f32>(0.0, 220.0 + 120.0*cos(time), 500.0), 100.0);
    // let s3 = sdSphere(P, vec3<f32>(400.0, 120.0, 120.0), 100.0 * (0.75 + 0.25*sin(time)));

    let s1 = sdSphere(P, vec3<f32>(-220.0, 50.0*cos(time), 200.0), 200.0);
    let s2 = sdSphere(P, vec3<f32>(0.0, 50.0*sin(time), 200.0), 200.0);
    let s3 = sdSphere(P, vec3<f32>(220.0, 50.0*cos(time), 200.0), 200.0);

    // let s = min(s1, min(s2, s3));
    let s = opSmoothUnion(s1, opSmoothUnion(s2, s3, 50.0), 50.0);

    let f = P.y + 350.0; // floor

    // return s;
    return min(s, f);
}


// normal
let DN = vec2<f32>(0.01, 0.0);

fn getNormal(P: vec3<f32>) -> vec3<f32> {
    return normalize( vec3<f32>(sdMap(P+DN.xyy), sdMap(P+DN.yxy), sdMap(P+DN.yyx)) - sdMap(P) );
}


// ray marching
let START_DIST = 1e-2; // start with a reasonable offset from surface dist
let SURFACE_DIST = 1e-4;
let MAX_DEPTH = 2000.0;
let MAX_ITER = 512;

struct RayHit { P: vec3<f32>, dist: f32 };

fn ray_march(Ro: vec3<f32>, Rd: vec3<f32>) -> RayHit {

    var dist = START_DIST;
    var P = vec3<f32>(0.0, 0.0, 0.0);
    var i = 0;

    loop {
        P = Ro + dist * Rd;
        let d = sdMap(P);
        dist += d;
        i += 1;
        if (dist > MAX_DEPTH) { break; }
        else if (abs(d) < SURFACE_DIST || i == MAX_ITER) {
            return RayHit(P, dist);
        }
    }

    return RayHit(vec3<f32>(0.0, 0.0, 0.0), -1.0);
}

// camera
let cd = 500.0; // half camera dimensions
let Co = vec3<f32>(0.0, 500.0, -2500.0); // camera origin

// light direction
let Ld = vec3<f32>(-0.3, -1.0, 0.5); // light direction
let LL = vec2<f32>(0.02, 0.10); // light levels (min, min lit)


// main
@fragment
fn fs_main(in: VertexData) -> @location(0) vec4<f32> {

    // constants
    let Ro = vec3<f32>(in.p * vec2<f32>(viewport.z * cd, cd) /*+ Co.xy*/, 0.0); // ray origin
    let Rd = normalize(Ro - Co); // ray direction

    // ray marching
    let H = ray_march(Ro, Rd);

    if (H.dist == -1.0) { // didn't hit anywhere
        return vec4<f32>(0.3, 0.0, 0.0, 1.0);
    }

    // lighting
    let Ln = normalize(Ld);

    let N = getNormal(H.P);
    var lf = dot(-N, Ln);
    var hl = 0.0; // highlights

    if (lf > 0.0) {
        lf = mix(LL.y, 1.0, lf);

        let S = ray_march(H.P, -Ln); // to Sun

        if (S.dist != -1.0) {
            lf = mix(LL.y, lf, min(S.dist/1000.0, 1.0)); ;
        }
        else { // highlight
            let Lr = Ln - 2.0*dot(Ln, N) * N;
            hl = pow(max(dot(Rd, -Lr), 0.0), 5.0) * 0.15;
        }
    }
    else {
        lf = mix(LL.x, LL.y, 1.0+lf);
    }

    hl += pow(1.0 - abs(dot(Rd, N)), 5.0) * 0.05; // edge reflection

    // color
    var color = vec3<f32>(1.0, 0.0, 0.0);
    color += vec3<f32>(hl);
    color *= lf;

    return vec4<f32>(color, 1.0);
}
