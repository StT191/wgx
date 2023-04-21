

// vertex and camera
struct VertexData {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) Co: vec3<f32>,
    @location(1) @interpolate(linear) Ro: vec3<f32>,
    @location(2) @interpolate(flat) Ln: vec3<f32>,
};

@vertex
fn vs_main(@location(0) p: vec2<f32>) -> VertexData {

    let Cn = normalize(Cd);
    let Co = Cp - cDist * Cn;
    let Ln = normalize(Ld);

    let sinX = Cn.y;
    let cosX = 1.0 - Cn.y;
    let sinY = -Cn.x / cosX;
    let cosY = 1.0 - sinY;

    let r = p * vec2<f32>(viewport.z * cDim, cDim);

    let Ro = vec3<f32>(
        r.x*cosY + r.y*sinX*sinY,
        r.y*cosX,
        r.x*sinY - r.y*sinX*cosY,
    ) + Cp;

    return VertexData(vec4<f32>(p, 0.0, 1.0), Co, Ro, Ln);
}



// normal
const dN = 0.01;

const dN1 = vec3<f32>( 1.0, -1.0, -1.0);
const dN2 = vec3<f32>(-1.0, -1.0,  1.0);
const dN3 = vec3<f32>(-1.0,  1.0, -1.0);
const dN4 = vec3<f32>( 1.0,  1.0,  1.0);

fn getNormal(P: vec3<f32>) -> vec3<f32> {
    return normalize(
        dN1 * sdMap(P + dN*dN1) +
        dN2 * sdMap(P + dN*dN2) +
        dN3 * sdMap(P + dN*dN3) +
        dN4 * sdMap(P + dN*dN4)
    );
}


// ray marching
const START_DIST = 1e-2; // start with a reasonable offset from surface dist
const SURFACE_DIST = 1e-4;
// const MAX_DEPTH = 3000.0; // declared externally
// const MAX_ITER = 64; // declared externally

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



// lighting
fn highlight(Rd: vec3<f32>, N: vec3<f32>, Ln: vec3<f32>) -> f32 {
    let Lr = Ln - 2.0*dot(Ln, N) * N;
    return pow(max(dot(Rd, -Lr), 0.0), hlPow) * hL;
}


// main
@fragment
fn fs_main(in: VertexData) -> @location(0) vec4<f32> {

    // ray marching
    let Rd = normalize(in.Ro - in.Co); // ray direction
    let H = ray_march(in.Ro, Rd);

    if (H.dist == -1.0) { // didn't hit anywhere
        return bgColor;
    }

    // lighting
    let N = getNormal(H.P);

    var lf = dot(-N, in.Ln);
    var hl = 0.0; // highlights

    if (lf > 0.0) { // in sun
        lf = mix(LL.y, 1.0, lf);

        if shDist > 0.0 {
            let S = ray_march(H.P, -in.Ln); // shadow dist

            if (S.dist != -1.0) {
                lf = mix(LL.y, lf, min(S.dist/shDist, 1.0)); // in shadow
            }
            else if (hL > 0.0) { // highlight
                hl = highlight(Rd, N, in.Ln);
            }
        }
        else if (hL > 0.0) { // highlight
            hl = highlight(Rd, N, in.Ln);
        }
    }
    else {
        lf = mix(LL.x, LL.y, 1.0+lf);
    }

    // color
    let color = lf * Color + vec3<f32>(hl);

    return vec4<f32>(color, 1.0);
}