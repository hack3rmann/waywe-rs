#version 460

layout(push_constant) uniform vec2 resolution;

layout(set = 0, binding = 0) uniform texture2D video_y_plane;
layout(set = 0, binding = 1) uniform texture2D video_uv_plane;
layout(set = 0, binding = 2) uniform sampler video_sampler;

in vec2 position;
out vec4 surface_color;

vec3 yuv_to_rgb(float y, float u, float v) {
    u -= 0.5;
    v -= 0.5;
    return vec3(
        y + (1.403 * v),
        y - (0.344 * u) - (0.714 * v),
        y + (1.770 * u)
    );
}

void main() {
    ivec2 video_size = textureSize(sampler2D(video_y_plane, video_sampler), 0);
    float scale_factor = resolution.x * float(video_size.y) / (resolution.y * float(video_size.x));

    vec2 scaled_position = vec2(scale_factor * position.x, position.y);

    vec2 texture_coordinates = 0.5 * scaled_position + 0.5;
    texture_coordinates.y = 1.0 - texture_coordinates.y;

    float y = texture(sampler2D(video_y_plane, video_sampler), texture_coordinates).r;
    vec2 uv = texture(sampler2D(video_uv_plane, video_sampler), texture_coordinates).rg;

    surface_color.rgb = yuv_to_rgb(y, uv.x, uv.y);
    surface_color.a = 1.0;

    surface_color.rgb = vec3(
        pow(surface_color.r, 1.0 / 0.4545),
        pow(surface_color.g, 1.0 / 0.4545),
        pow(surface_color.b, 1.0 / 0.4545)
    );
}
