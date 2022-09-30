
// normal
let DN = vec2<f32>(0.01, 0.0);

fn getNormal(P: vec3<f32>) -> vec3<f32> {
    return normalize( vec3<f32>(sdMap(P+DN.xyy), sdMap(P+DN.yxy), sdMap(P+DN.yyx)) - sdMap(P) );
}


// ray marching
let START_DIST = 1e-2; // start with a reasonable offset from surface dist
let SURFACE_DIST = 1e-4;
// let MAX_DEPTH = 3000.0; // declared externally
// let MAX_ITER = 256; // declared externally

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

