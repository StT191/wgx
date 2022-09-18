
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
let pi = 3.141592653589793;
let pi2 = 6.283185307179586;
// let sqrt2 = 1.4142135623730951;


// uniform
@group(0) @binding(0) var<uniform> viewport: vec3<f32>;
@group(0) @binding(1) var<uniform> scale: f32;
// @group(0) @binding(2) var<uniform> time: f32;

var<push_constant> time: f32; // time in secs


// sdf map
let MAX_DEPTH = 3000.0;

fn sdMap(P: vec3<f32>) -> f32 {

    let h = P.y
        + (
            10.0*sin(P.z/500.0 + time) +
            4.0*sin(P.z/80.0 + 0.0) +
            0.5*sin(P.z/6.0 + 0.0)
        )
        * (
            15.0*cos(P.x/300.0 - pi - time) +
            5.0*cos(P.x/120.0 - 0.0) +
            0.5*cos(P.x/4.0 - 0.0)
        )
        / 3.0
        * max(1.0 - 1.05*P.z/MAX_DEPTH, 0.0)
    ; // floor

    // return s;
    return h;
}


// normal
let DN = vec2<f32>(0.01, 0.0);

fn getNormal(P: vec3<f32>) -> vec3<f32> {
    return normalize( vec3<f32>(sdMap(P+DN.xyy), sdMap(P+DN.yxy), sdMap(P+DN.yyx)) - sdMap(P) );
}


// ray marching
let START_DIST = 1e-2; // start with a reasonable offset from surface dist
let SURFACE_DIST = 1e-4;
let MAX_ITER = 96;

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
            return RayHit(P, d);
        }
    }

    return RayHit(vec3<f32>(0.0, 0.0, 0.0), -1.0);
}

// camera
let cd = 500.0; // half camera dimensions
let Co = vec3<f32>(0.0, 500.0, -2500.0); // camera origin

// light direction
let Ld = vec3<f32>(-0.3, -1.0, -1.5); // light direction
let LL = vec2<f32>(0.02, 0.10); // light levels (min, min lit)
let la = 0.03; // ambient light


// main
@fragment
fn fs_main(in: VertexData) -> @location(0) vec4<f32> {

    // constants
    let Ro = vec3<f32>(in.p * vec2<f32>(viewport.z * cd, cd) /*+ Co.xy*/, 0.0); // ray origin
    let Rd = normalize(Ro - Co); // ray direction

    // ray marching
    let H = ray_march(Ro, Rd);

    if (H.dist == -1.0) { // didn't hit anywhere
        return vec4<f32>(0.01, 0.5, 0.8, 1.0);
    }

    // lighting
    let Ln = normalize(Ld);

    let N = getNormal(H.P);
    var lf = dot(-N, Ln);
    var hl = 0.0; // highlights

    if (lf > 0.0) {
        lf = mix(LL.y, 1.0, lf);

        // let S = ray_march(H.P, -Ln); // to Sun

        // if (S.dist != -1.0) {
        //     lf = mix(LL.y, lf, min(S.dist/1000.0, 1.0)); ;
        // }
        // else { // highlight
            let Lr = Ln - 2.0*dot(Ln, N) * N;
            hl = pow(max(dot(Rd, -Lr), 0.0), 50.0) * 0.15;
        // }
    }
    else {
        lf = mix(LL.x, LL.y, 1.0+lf);
    }

    hl += pow(1.0 - abs(dot(Rd, N)), 50.0) * 0.05; // edge reflection

    // color
    var color = vec3<f32>(1.0, 0.0, 0.0);
    color += vec3<f32>(hl);
    color *= lf;

    return vec4<f32>(color, 1.0);
}
