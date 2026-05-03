#version 460

layout(push_constant) uniform Push { 
  vec2 resolution; 
} push;

layout(set = 0, binding = 0) uniform texture2D video_y_plane;
layout(set = 0, binding = 1) uniform texture2D video_uv_plane;
layout(set = 0, binding = 2) uniform sampler video_sampler;

layout (location = 0) in vec2 position;
layout (location = 0) out vec4 surface_color;

vec3 yuv_to_rgb(float y, float u, float v) {
    return vec3(
        y + 1.5748 * v,
        y - 0.1873 * u - 0.4681 * v,
        y + 1.8556 * u
    );
}

const vec3 LUMA_DIRECTION = vec3(0.2126, 0.7152, 0.0772);

void main() {
    ivec2 video_size = textureSize(sampler2D(video_y_plane, video_sampler), 0);

    float video_aspect_ratio = float(video_size.x) / float(video_size.y);
    float screen_aspect_ratio = push.resolution.x / push.resolution.y;

    float scale_factor = push.resolution.x * float(video_size.y) / (push.resolution.y * float(video_size.x));
    vec2 scaled_position = vec2(scale_factor * position.x, position.y);

    if (video_aspect_ratio < screen_aspect_ratio) {
        scaled_position /= scale_factor;
    }

    vec2 texture_coordinates = 0.5 * scaled_position + 0.5;
    texture_coordinates.y = 1.0 - texture_coordinates.y;

    float y = texture(sampler2D(video_y_plane, video_sampler), texture_coordinates).r;
    vec2 uv = texture(sampler2D(video_uv_plane, video_sampler), texture_coordinates).rg;

    // Expand limited range to full range
    y = (y - 16.0/255.0) * (255.0 / (235.0 - 16.0));
    uv = (uv - vec2(128.0/255.0)) * (255.0 / (240.0 - 16.0));

    surface_color.rgb = yuv_to_rgb(y, uv.x, uv.y);
    surface_color.a = 1.0;

    surface_color.rgb = vec3(
        pow(surface_color.r, 1.0 / 0.4545),
        pow(surface_color.g, 1.0 / 0.4545),
        pow(surface_color.b, 1.0 / 0.4545)
    );
}
