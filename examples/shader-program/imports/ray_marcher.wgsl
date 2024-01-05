

// vertex and camera
struct VertexData {
    @builtin(position) position: vec4f,
    @location(0) @interpolate(flat) Co: vec3f,
    @location(1) @interpolate(perspective) Ro: vec3f,
    @location(2) @interpolate(flat) Ln: vec3f,
};

@vertex
fn vs_main(@location(0) p: vec2f) -> VertexData {

    let Cn = normalize(Cd);
    let Co = Cp - cDist * Cn;
    let Ln = normalize(Ld);

    let sinX = Cn.y;
    let cosX = 1.0 - Cn.y;
    let sinY = -Cn.x / cosX;
    let cosY = 1.0 - sinY;

    let r = p * vec2f(view.z * cDim, cDim);

    let Ro = vec3f(
        r.x*cosY + r.y*sinX*sinY,
        r.y*cosX,
        r.x*sinY - r.y*sinX*cosY,
    ) + Cp;

    return VertexData(vec4f(p, 0.0, 1.0), Co, Ro, Ln);
}


// rays
struct RayField { dist: f32, color: vec4f };


// normal
const dN = 0.01;

const dN1 = vec3f( 1.0, -1.0, -1.0);
const dN2 = vec3f(-1.0, -1.0,  1.0);
const dN3 = vec3f(-1.0,  1.0, -1.0);
const dN4 = vec3f( 1.0,  1.0,  1.0);

fn getNormal(P: vec3f) -> vec3f {
    return normalize(
        dN1 * sdMap(P + dN*dN1, false).dist +
        dN2 * sdMap(P + dN*dN2, false).dist +
        dN3 * sdMap(P + dN*dN3, false).dist +
        dN4 * sdMap(P + dN*dN4, false).dist
    );
}


// ray marching
// const START_DIST = 1e-2; // start with a reasonable offset from surface dist // declared externally
// const SURFACE_DIST = 1e-4; // declared externally
// const MAX_DEPTH = 3000.0; // declared externally
// const MAX_ITER = 64; // declared externally

struct RayHit { P: vec3f, dist: f32, color: vec4f };

fn ray_march(Ro: vec3f, Rd: vec3f, map_color: bool) -> RayHit {

    var abs_dist = START_DIST;
    var i = 0;

    loop {
        let P = Ro + abs_dist * Rd;
        let field = sdMap(P, map_color);
        abs_dist += field.dist;
        i += 1;
        if (abs_dist > MAX_DEPTH) { break; }
        else if (abs(field.dist) < SURFACE_DIST || i == MAX_ITER) {
            return RayHit(P, abs_dist, field.color);
        }
    }

    return RayHit(vec3f(0.0, 0.0, 0.0), -1.0, vec4f(0.0, 0.0, 0.0, 0.0));
}



// lighting
fn highlight(Rd: vec3f, N: vec3f, Ln: vec3f) -> f32 {
    let Lr = Ln - 2.0*dot(Ln, N) * N;
    return pow(max(dot(Rd, -Lr), 0.0), hlPow) * hL;
}


// main
@fragment
fn fs_main(in: VertexData) -> @location(0) vec4f {

    // ray marching
    let Rd = normalize(in.Ro - in.Co); // ray direction
    let H = ray_march(in.Ro, Rd, true);

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
            let S = ray_march(H.P, -in.Ln, false); // shadow dist

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
    let color = lf * H.color.rgb + vec3f(hl);

    return vec4f(color, H.color.a);
}