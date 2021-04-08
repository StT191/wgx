#version 450

// in
layout(location = 0) in vec2 a_Pos;
layout(location = 1) in vec4 a_Color;

// uniform
layout(set = 0, binding = 0) uniform GlobalP { mat4 u_ProjectionMatrix; };
// layout(set = 0, binding = 1) uniform GlobalS { mat4 u_ScaleMatrix; };
// layout(set = 0, binding = 1) uniform GlobalV { vec2 u_Dim; };


// out
layout(location = 0) out vec4 v_Color;
layout(location = 1) out vec2 v_NormCoord;
// layout(location = 2) out vec2 v_Dim;


// function
void main() {
    v_Color = a_Color;
    v_NormCoord = a_Pos;

    // vec4 xdim = u_ScaleMatrix * vec4(a_Pos, 0.0, 1.0);
    // v_Dim = vec2(xdim.x, xdim.y);

    gl_Position = u_ProjectionMatrix * vec4(a_Pos, 0.0, 1.0);
}