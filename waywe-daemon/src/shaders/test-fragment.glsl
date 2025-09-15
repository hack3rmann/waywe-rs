#version 460

layout(push_constant) uniform struct PushConst {
    float time;
} push;

in vec2 position;
out vec4 surface_color;

void main() {
    surface_color = vec4(position, 0.5 * sin(push.time) + 0.5, 1.0);
}
