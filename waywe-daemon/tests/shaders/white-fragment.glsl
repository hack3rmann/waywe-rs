#version 460

in vec3 vertex_color;
out vec4 surface_color;

void main() {
    surface_color = vec4(vertex_color, 1.0);
}
