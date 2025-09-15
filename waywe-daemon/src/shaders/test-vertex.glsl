#version 460

layout(location = 0) in vec3 vertex_position;

out vec2 position;

void main() {
    position = vertex_position.xy;
    gl_Position = vec4(vertex_position, 1.0);
}
