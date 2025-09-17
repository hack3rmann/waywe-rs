#version 460

layout(push_constant) uniform struct PushConst {
    mat4 model;
    float time;
} push;

layout(location = 0) in vec3 vertex_position;

out vec2 position;

void main() {
    position = vertex_position.xy;
    gl_Position = push.model * vec4(vertex_position, 1.0);
}
