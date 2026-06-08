#version 460

layout(location = 0) in vec2 vertex_position;
layout(location = 0) out vec2 fragCoord;

layout(push_constant) uniform PushConst {
    vec2 resolution;
    float time;
} push;

void main() {
    fragCoord = push.resolution * (0.5 * vertex_position + 0.5);
    gl_Position = vec4(vertex_position, 0.0, 1.0);
}
