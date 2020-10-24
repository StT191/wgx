#version 450

// input
layout(location = 0) in vec3 a_Pos;

// output
layout(location = 0) out vec2 v_TexCoord;

// function
void main() {
    v_TexCoord = vec2(0.0, 0.0);
    gl_PointSize = 44.0;
    gl_Position = vec4(a_Pos, 1.0);
}