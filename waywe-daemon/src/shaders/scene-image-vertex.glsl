#version 460

layout(push_constant) uniform struct PushConst {
    mat4 mvp;
    float time;
} push;

layout(location = 0) in vec2 vertex_position;

out vec2 uv;

void main() {
    gl_Position = push.mvp * vec4(vertex_position, 0.0, 1.0);
    uv = 0.5 * vertex_position + 0.5;
    uv.y = 1.0 - uv.y;
}
