#version 460

layout(push_constant) uniform struct PushConst {
    vec2 resolution;
    uint transparency_color;
} push;

layout(set = 0, binding = 0) uniform texture2D image;
layout(set = 0, binding = 1) uniform sampler image_sampler;

in vec2 position;
out vec4 surface_color;

vec4 unpack_color(uint color) {
    return vec4(
        float((color >> 24) & 0xFF) / 255.0,
        float((color >> 16) & 0xFF) / 255.0,
        float((color >> 8) & 0xFF) / 255.0,
        float((color >> 0) & 0xFF) / 255.0
    );
}

void main() {
    ivec2 image_size = textureSize(sampler2D(image, image_sampler), 0);

    float image_aspect_ratio = float(image_size.x) / float(image_size.y);
    float screen_aspect_ratio = push.resolution.x / push.resolution.y;

    float scale_factor = push.resolution.x * float(image_size.y) / (push.resolution.y * float(image_size.x));
    vec2 scaled_position = vec2(scale_factor * position.x, position.y);

    if (image_aspect_ratio < screen_aspect_ratio) {
        scaled_position /= scale_factor;
    }

    vec2 texture_coordinates = 0.5 * scaled_position + 0.5;
    texture_coordinates.y = 1.0 - texture_coordinates.y;

    vec4 sample_color = texture(sampler2D(image, image_sampler), texture_coordinates);

    surface_color.rgb = mix(
        unpack_color(push.transparency_color).rgb,
        sample_color.rgb,
        sample_color.a
    );
    surface_color.a = 1.0;

    surface_color.rgb = vec3(
        pow(surface_color.r, 1.0 / 0.4545),
        pow(surface_color.g, 1.0 / 0.4545),
        pow(surface_color.b, 1.0 / 0.4545)
    );
}
