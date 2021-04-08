#version 450

// in
layout(location = 0) in vec3 a_Pos;
layout(location = 1) in vec2 a_TexCoord;

// out
layout(location = 0) out vec2 v_TexCoord;

// function
void main() {
    v_TexCoord = a_TexCoord;
    gl_Position = vec4(a_Pos, 1.0);
}