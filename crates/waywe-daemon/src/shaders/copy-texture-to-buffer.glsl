#version 460

layout (local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

layout (set = 0, binding = 0) uniform texture2D image;
layout (set = 0, binding = 1) uniform sampler image_sampler;
layout (set = 0, binding = 2, std430) buffer Buffer {
    uint out_bytes[];
};

void main() {
    ivec2 coord = ivec2(gl_GlobalInvocationID.xy);
    vec4 texel = texelFetch(sampler2D(image, image_sampler), coord, 0);

    uvec4 bytes = uvec4(round(texel * 255.0));
    uint i = coord.y * textureSize(sampler2D(image, image_sampler), 0).x + coord.x;

    out_bytes[i] = bytes.r | (bytes.g << 8) | (bytes.b << 16) | (bytes.a << 24);
}
