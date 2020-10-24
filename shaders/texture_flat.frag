#version 450

// input
layout(set = 0, binding = 0) uniform texture2D t_Color;
layout(set = 0, binding = 1) uniform sampler s_Color;

// output
layout(location = 0) in vec2 v_TexCoord;
layout(location = 0) out vec4 color;

// function
void main() {
    color = texture(sampler2D(t_Color, s_Color), v_TexCoord);
}