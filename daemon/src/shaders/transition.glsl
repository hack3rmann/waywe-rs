#version 460

layout(push_constant) uniform struct Push {
    vec2 centre;
    float radius;
} push;

layout(set = 0, binding = 0) uniform texture2D from_image;
layout(set = 0, binding = 1) uniform texture2D to_image;
layout(set = 0, binding = 2) uniform sampler image_sampler;

in vec2 position;
out vec4 surface_color;

void main() {
    ivec2 image_size = textureSize(sampler2D(from_image, image_sampler), 0);
    float aspect_ratio = float(image_size.x) / float(image_size.y);

    vec2 texture_coordinates = 0.5 * position + 0.5;
    texture_coordinates.y = 1.0 - texture_coordinates.y;

    vec2 offset = position;
    offset.x *= aspect_ratio;
    offset -= push.centre;

    if (dot(offset, offset) < push.radius * push.radius) {
        surface_color.rgb = texture(sampler2D(to_image, image_sampler), texture_coordinates).rgb;
    } else {
        surface_color.rgb = texture(sampler2D(from_image, image_sampler), texture_coordinates).rgb;
    }

    surface_color.a = 1.0;
}
