#version 450

// in
layout(location = 0) in vec3 a_Pos;
layout(location = 1) in vec2 a_TexCoord;


// uniform
layout(set = 0, binding = 2) uniform Globals { mat4 u_ProjectionMatrix; };


// out
layout(location = 0) out vec2 v_TexCoord;


// function
void main() {
    v_TexCoord = a_TexCoord;
    gl_Position = u_ProjectionMatrix * vec4(a_Pos, 1.0);
}