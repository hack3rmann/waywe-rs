#version 460

layout(push_constant) uniform Push {
    float progress;
} push;

layout(set = 0, binding = 0) uniform texture2D from_image;
layout(set = 0, binding = 1) uniform texture2D to_image;
layout(set = 0, binding = 2) uniform sampler image_sampler;

layout (location = 0) in vec2 position;
layout (location = 0) out vec4 surface_color;

void main() {
    vec2 texture_coordinates = 0.5 * position + 0.5;
    texture_coordinates.y = 1.0 - texture_coordinates.y;

    vec3 color_from = texture(sampler2D(from_image, image_sampler), texture_coordinates).rgb;
    vec3 color_to = texture(sampler2D(to_image, image_sampler), texture_coordinates).rgb;

    surface_color.rgb = mix(color_from, color_to, push.progress);

    surface_color.a = 1.0;
}
