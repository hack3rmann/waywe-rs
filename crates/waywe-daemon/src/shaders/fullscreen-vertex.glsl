#version 460

layout(location = 0) in vec2 vertex_position;

layout(location = 0) out vec2 position;

void main() {
    position = vertex_position;
    gl_Position = vec4(vertex_position, 0.0, 1.0);
}
