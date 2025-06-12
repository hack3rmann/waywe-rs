#version 460

layout(push_constant) uniform float time;

layout(set = 0, binding = 0) uniform texture2D image;
layout(set = 0, binding = 1) uniform sampler image_sampler;

in vec2 position;
out vec4 surface_color;

void main() {
    ivec2 image_size = textureSize(sampler2D(image, image_sampler), 0);
    float aspect_ratio = float(image_size.x) / float(image_size.y);

    vec2 texture_coordinates = 0.5 * position + 0.5;
    texture_coordinates.y = 1.0 - texture_coordinates.y;

    vec2 offset = vec2(texture_coordinates) - vec2(0.5);
    offset.x *= aspect_ratio;
    float radius = log(1.0 + time);

    if (dot(offset, offset) < radius * radius) {
        discard;
    }

    surface_color.rgb = texture(sampler2D(image, image_sampler), texture_coordinates).rgb;
    surface_color.a = 1.0;
}
