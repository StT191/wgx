
// locals
struct Matrix { m: mat4x4<f32> };
struct Vec2 { v: vec2<f32> };

@group(0) @binding(0) var<uniform> world: Matrix;
@group(0) @binding(1) var<uniform> clip: Matrix;
@group(0) @binding(2) var<uniform> viewport: Vec2;


struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) ty: f32,
    @location(1) @interpolate(flat) color: vec4<f32>,
    @location(2) @interpolate(perspective) E: vec2<f32>,
    // @location(3) @interpolate(flat) prj: mat4x4<f32>,
    @location(3) @interpolate(flat) p0: vec4<f32>,
    @location(4) @interpolate(flat) p1: vec4<f32>,
    @location(5) @interpolate(flat) p2: vec4<f32>,
    @location(6) @interpolate(flat) p3: vec4<f32>,
};


// untility functions
fn from_vecs(O:vec3<f32>, X:vec3<f32>, Y:vec3<f32>, Z:vec3<f32>) -> mat4x4<f32> {
    return mat4x4<f32>(vec4<f32>(X, 0.0), vec4<f32>(Y, 0.0), vec4<f32>(Z, 0.0), vec4<f32>(O, 1.0));
}
fn from_translation(x:f32, y:f32, z:f32) -> mat4x4<f32> {
    return mat4x4<f32>(vec4<f32>(1.0, 0.0, 0.0, 0.0), vec4<f32>(0.0, 1.0, 0.0, 0.0), vec4<f32>(0.0, 0.0, 1.0, 0.0), vec4<f32>(x, y, z, 1.0));
}
fn from_scale(x:f32, y:f32, z:f32) -> mat4x4<f32> {
    return mat4x4<f32>(vec4<f32>(x, 0.0, 0.0, 0.0), vec4<f32>(0.0, y, 0.0, 0.0), vec4<f32>(0.0, 0.0, z, 0.0), vec4<f32>(0.0, 0.0, 0.0, 1.0));
}
fn normal_2d(v:vec2<f32>) -> vec2<f32> {
    return vec2<f32>(v.y, -v.x);
}
fn homogen_2d(v:vec4<f32>) -> vec2<f32> {
    return vec2<f32>(v.x/v.w, v.y/v.w);
}
fn homogen_3d(v:vec4<f32>) -> vec3<f32> {
    return vec3<f32>(v.x/v.w, v.y/v.w, v.z/v.w);
}


let Z0 = vec3<f32>(0.0, 0.0, 0.0);

@vertex
fn vs_main(
    @builtin(vertex_index) i: u32,
    @location(0) P0: vec3<f32>,
    @location(1) P1: vec3<f32>,
    @location(2) P2: vec3<f32>,
    @location(3) ty: f32,
    @location(4) color: u32,
) -> VertexOutput {
    var out: VertexOutput;

    out.ty = ty;
    out.color = unpack4x8unorm(color);

    if (ty == 0.0) { // triangle
        switch (i) {
            case 0u: { out.position = clip.m * world.m * vec4<f32>(P0, 1.0); break; }
            case 1u: { out.position = clip.m * world.m * vec4<f32>(P1, 1.0); break; }
            case 2u: { out.position = clip.m * world.m * vec4<f32>(P2, 1.0); break; }
            default: {}
        }

        return out;
    }

    let O = homogen_3d(world.m * vec4<f32>(P1, 1.0));
    let X = homogen_3d(world.m * vec4<f32>(P0, 1.0)) - O;
    let Y = homogen_3d(world.m * vec4<f32>(P2, 1.0)) - O;

    if (ty > 0.0) {
        let q = sqrt(2.0);

        switch (i) {
            case 0u: { out.position = clip.m * vec4<f32>(O+q*X, 1.0); out.E = vec2<f32>(q,   0.0); break; }
            case 1u: { out.position = clip.m * vec4<f32>(O,     1.0); out.E = vec2<f32>(0.0, 0.0); break; }
            case 2u: { out.position = clip.m * vec4<f32>(O+q*Y, 1.0); out.E = vec2<f32>(0.0,   q); break; }
            default: {}
        }
    }
    else {
        switch (i) { // negative, color the outside
            case 0u: { out.position = clip.m * vec4<f32>(O+X, 1.0); out.E = vec2<f32>(0.0, 1.0); break; }
            case 1u: { out.position = clip.m * vec4<f32>(O,   1.0); out.E = vec2<f32>(1.0, 1.0); break; }
            case 2u: { out.position = clip.m * vec4<f32>(O+Y, 1.0); out.E = vec2<f32>(1.0, 0.0); break; }
            default: {}
        }
    }

    // get unit circle to screen pixel matrix
    let unit_to_pix = from_scale(viewport.v.x/2.0, viewport.v.y/2.0, 1.0) * clip.m * from_vecs(O, X, Y, Z0);

    let O = homogen_2d(unit_to_pix * vec4<f32>(0.0, 0.0, 0.0, 1.0)); // origin

    let prj = from_translation(-O.x, -O.y, 0.0) * unit_to_pix;

    out.p0 = prj[0]; out.p1 = prj[1]; out.p2 = prj[2]; out.p3 = prj[3];

    return out;
}


let g:f32 = 1.0; // edge-blur


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    let prj = mat4x4(in.p0, in.p1, in.p2, in.p3);

    if (in.ty == 0.0) { // triangle
        return in.color;
    }

    let e = length(in.E);

    if (e > 1.0) {
        if (in.ty > 0.0) {
            discard;
        }
        else {
            return in.color; // color outside
        }
    }


    // get vectors

    let R = homogen_2d(prj * vec4<f32>(in.E, 0.0, 1.0)); // R in pixel space

    let Er = in.E * 1.0/e; // R at rim in unit circle
    let Rr = homogen_2d(prj * vec4<f32>(Er, 0.0, 1.0)); // R at rim in pixel space

    let dR = Rr - R;
    let dr = length(dR);

    let En = normal_2d(Er);


    // types

    if (in.ty <= 1.0) {

        let T = homogen_2d(prj * vec4<f32>(Er + En, 0.0, 1.0)) - Rr; // tangent in pixel space

        let ds = dr * sin(acos(dot(dR, T) / (dr*length(T)))); // skew corrected pixel distance to rim


        // types

        if (in.ty < 0.0) { // color outside

            if (ds < g) {
                return vec4<f32>(in.color.rgb, in.color.a * (1.0 - ds/g)); // edge blur
            }
            else {
                discard;
            }
        }


        if (ds < g) {
            return vec4<f32>(in.color.rgb, in.color.a * ds/g); // edge blur
        }


        if (in.ty == 1.0) {
            return in.color; // color fully inside
        }

        else { // 0.0 < in.ty < 1.0

            let h = 1.0 - in.ty; // portion of unit circle radius

            if (e < h) {

                // if (Er.x < 0.05 || Er.y < 0.05) {
                //     return vec4<f32>(0.0, 0.0, 1.0, 1.0);
                // }

                discard;
            }
            else {
                let dv = (e - h) / (1.0 - e) * ds;

                if (dv < g) {
                    return vec4<f32>(in.color.rgb, in.color.a * dv/g);
                }

                return in.color;
            }
        }
    }

    else { // in.ty > 1.0

        // constant edge-distance in pixel

        var ds = dr;

        let dn = sqrt(1.0 - e*e);

        var Eh = Er; // point on unit circle
        var Rh = Rr; // ... in pixel space
        var dRh = dR; // delta vec in pixel space
        var drh = dr; // delta length

        for (var j = 0; j != 3; j = j + 1) {

            if (j != 0) {
                if (j == 1) { Eh = in.E + dn * En; }
                else        { Eh = in.E - dn * En; }

                Rh = homogen_2d(prj * vec4<f32>(Eh, 0.0, 1.0));
                dRh = Rh - R;
                drh = length(dRh);
            }

            for (var i = 0; i != 2; i = i + 1) {

                let En = normal_2d(Eh);

                let T = homogen_2d(prj * vec4<f32>(Eh + En, 0.0, 1.0)) - Rh; // tangent in pixel space
                let t = length(T);

                // reassign
                Eh = normalize(Eh - En * (dot(dRh, T) / (t*t))); // dl = cos_t * drh/t

                Rh = homogen_2d(prj * vec4<f32>(Eh, 0.0, 1.0));
                dRh = Rh - R;
                drh = length(dRh);
            }

            if (drh < ds) {
                ds = drh;
            }
        }


        if (ds < g) {
            return vec4<f32>(in.color.rgb, in.color.a * ds/g); // edge blur
        }

        let b = in.ty; // rim width in pixel space

        if (ds > b) {

            // if (Er.x < 0.05 || Er.y < 0.05) {
            //     return vec4<f32>(0.0, 0.0, 1.0, 1.0);
            // }

            discard;
        }
        else {
            let dv = b - ds;

            if (dv < g) {
                return vec4<f32>(in.color.rgb, in.color.a * dv/g);
            }

            return in.color;
        }
    }


    return in.color;
}