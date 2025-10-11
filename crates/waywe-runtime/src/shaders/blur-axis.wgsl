@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var output: texture_storage_2d<bgra8unorm, write>;

var<push_constant> axis: u32;

fn invert_gamma(sample: vec3f) -> vec3f {
    return pow(sample, vec3f(2.2));
}

fn correct_gamma(sample: vec3f) -> vec3f {
    return pow(sample, vec3f(1.0 / 2.2));
}

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3u) {
    const offsets = array<vec2u, 4>(
        // NOTE(hack3rmann): using integer overflow to subtract one
        vec2u(0xFFFFFFFF, 0),
        vec2u(1, 0),
        vec2u(0, 0xFFFFFFFF),
        vec2u(0, 1),
    );

    let left = textureLoad(input, id.xy + offsets[0 + (axis << 1)], 0).rgb;
    let center = textureLoad(input, id.xy, 0).rgb;
    let right = textureLoad(input, id.xy + offsets[1 + (axis << 1)], 0).rgb;

    let average = correct_gamma(
        0.25 * invert_gamma(left) +
        0.5 * invert_gamma(center) +
        0.25 * invert_gamma(right)
    );

    textureStore(output, id.xy, vec4f(average, 1.0));
}
