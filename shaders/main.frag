#version 450

layout(set = 0, binding = 0) uniform texture2D t_Color;
layout(set = 0, binding = 1) uniform sampler s_Color;

layout(location = 0) in vec2 v_TexCoord;
layout(location = 0) out vec4 color;

void main() {
    // color = vec4(v_TexCoord/2, 0.0, 1.0);
    color = texture(sampler2D(t_Color, s_Color), v_TexCoord);
}
