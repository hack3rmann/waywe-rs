#version 460

layout(push_constant) uniform struct PushConst {
    vec2 resolution;
    float time;
} push;

layout(set = 0, binding = 0) uniform texture2D video;
layout(set = 0, binding = 1) uniform sampler video_sampler;

in vec2 position;
out vec4 surface_color;

void main() {
    vec2 texture_coordinates = 0.5 * position + 0.5;
    texture_coordinates.y = 1.0 - texture_coordinates.y;

    surface_color = texture(sampler2D(video, video_sampler), texture_coordinates);

    surface_color.rgb = vec3(
        pow(surface_color.r, 1.0 / 0.4545),
        pow(surface_color.g, 1.0 / 0.4545),
        pow(surface_color.b, 1.0 / 0.4545)
    );
}
