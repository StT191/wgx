#version 450

// binding
layout(set = 0, binding = 0) uniform Globals { vec4 u_Color; };

// output
layout(location = 0) out vec4 color;

// function
void main() {
    color = u_Color;
}
