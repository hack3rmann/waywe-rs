#version 460

layout(push_constant) uniform vec2 resolution;

layout(set = 0, binding = 0) uniform texture2D image;
layout(set = 0, binding = 1) uniform sampler image_sampler;

in vec2 position;
out vec4 surface_color;

void main() {
    ivec2 image_size = textureSize(sampler2D(image, image_sampler), 0);
    float scale_factor = resolution.x * float(image_size.y) / (resolution.y * float(image_size.x));

    vec2 scaled_position = vec2(scale_factor * position.x, position.y);

    vec2 texture_coordinates = 0.5 * scaled_position + 0.5;
    texture_coordinates.y = 1.0 - texture_coordinates.y;

    surface_color.rgb = texture(sampler2D(image, image_sampler), texture_coordinates).rgb;
    surface_color.a = 1.0;

    surface_color.rgb = vec3(
        pow(surface_color.r, 1.0 / 0.4545),
        pow(surface_color.g, 1.0 / 0.4545),
        pow(surface_color.b, 1.0 / 0.4545)
    );
}
