#version 460

layout(push_constant) uniform struct PushConst {
    mat4 model;
    float time;
    uint _padding[3];
} push;

layout(set = 0, binding = 0) uniform texture2D image;
layout(set = 0, binding = 1) uniform sampler image_sampler;

in vec2 uv;
out vec4 surface_color;

void main() {
    vec4 sample_color = texture(sampler2D(image, image_sampler), uv);
    surface_color = sample_color;
    surface_color.rgb = vec3(
        pow(surface_color.r, 1.0 / 0.4545),
        pow(surface_color.g, 1.0 / 0.4545),
        pow(surface_color.b, 1.0 / 0.4545)
    );
}
