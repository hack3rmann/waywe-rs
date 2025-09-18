#version 460

layout(push_constant) uniform struct PushConst {
    mat4 model;
    float time;
    uint _padding[3];
} push;

layout(set = 1, binding = 0) uniform texture2D image;
layout(set = 1, binding = 1) uniform sampler image_sampler;

in vec2 uv;
out vec4 surface_color;

void main() {
    vec4 sample_color = texture(sampler2D(image, image_sampler), uv);
    surface_color = sample_color;
}
