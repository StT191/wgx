#version 450

// uniform
layout(set = 0, binding = 1) uniform GlobalV { vec2 u_Dim; };

// in
layout(location = 0) in vec4 v_Color;
layout(location = 1) in vec2 v_NormCoord;
// layout(location = 2) in vec2 v_Dim;

// out
layout(location = 0) out vec4 color;

// function
void main() {

    float tn = sqrt(pow(v_NormCoord.x, 2.0) + pow(v_NormCoord.y, 2.0));

    if (tn > 1.0) discard;

    color = v_Color;

    float ts = sqrt(pow(v_NormCoord.x*u_Dim.x, 2.0) + pow(v_NormCoord.y*u_Dim.y, 2.0));

    float ds = ts/tn * (1.0 - tn);

    float g = 3.0;

    if (ds < g) {
        color.a *= ds / g;
    }
}