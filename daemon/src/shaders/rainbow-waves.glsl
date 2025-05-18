#version 460

layout(push_constant) uniform struct PushConst {
    vec2 resolution;
    float time;
} push;

in vec2 position;
out vec4 surface_color;

void main() {
    float i, d, z, r;

    for (surface_color *= i; i++ < 9e1; surface_color += (cos(z * 0.5 + push.time + vec4(0, 2, 4, 3)) + 1.3) / d / z) {
        vec3 p = z * normalize(vec3(2.0 * (0.5 * position + 0.5) * push.resolution, 0) - push.resolution.xyy);
        r = max(-++p, 0.0).y;
        p.y += r + r;

        for (d = 1.0; d < 3e1; d += d) {
            p.y += cos(p * d + 2. * push.time * cos(d) + z).x / d;
        }

        z += d = (0.1 * r + abs(p.y - 1.0) / (1.0 + 2.0 * r + r * r) + max(d = p.z + 3.0, -d * 0.1)) / 8.0;
    }

    surface_color = tanh(surface_color / 9e2);
}
