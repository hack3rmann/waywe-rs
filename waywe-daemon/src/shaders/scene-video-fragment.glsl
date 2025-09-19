#version 460

layout(push_constant) uniform struct PushConst {
    mat4 model;
    float time;
    uint _padding[3];
} push;

layout(set = 0, binding = 0) uniform texture2D video_y_plane;
layout(set = 0, binding = 1) uniform texture2D video_uv_plane;
layout(set = 0, binding = 2) uniform sampler video_sampler;

in vec2 uv;
out vec4 surface_color;

vec3 yuv_to_rgb(vec3 yuv) {
    yuv.y -= 0.5;
    yuv.z -= 0.5;
    return vec3(
        yuv.x + (1.403 * yuv.z),
        yuv.x - (0.344 * yuv.y) - (0.714 * yuv.z),
        yuv.x + (1.770 * yuv.y)
    );
}

const vec3 LUMA_DIRECTION = vec3(0.2126, 0.7152, 0.0772);

void main() {
    vec3 yuv = vec3(
        texture(sampler2D(video_y_plane, video_sampler), uv).r,
        texture(sampler2D(video_uv_plane, video_sampler), uv).rg
    );

    surface_color.rgb = yuv_to_rgb(yuv);
    surface_color.a = 1.0;

    surface_color.rgb = vec3(
        pow(surface_color.r, 1.0 / 0.4545),
        pow(surface_color.g, 1.0 / 0.4545),
        pow(surface_color.b, 1.0 / 0.4545)
    );
}
