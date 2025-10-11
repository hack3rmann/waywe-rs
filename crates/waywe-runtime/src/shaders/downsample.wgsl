@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var output: texture_storage_2d<bgra8unorm, write>;

fn invert_gamma(sample: vec3f) -> vec3f {
    return pow(sample, vec3f(2.2));
}

fn correct_gamma(sample: vec3f) -> vec3f {
    return pow(sample, vec3f(1.0 / 2.2));
}

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3u) {
    let ll = textureLoad(input, vec2u(2u * id.x, 2u * id.y), 0).rgb;
    let lh = textureLoad(input, vec2u(2u * id.x, 2u * id.y + 1u), 0).rgb;
    let hl = textureLoad(input, vec2u(2u * id.x + 1u, 2u * id.y), 0).rgb;
    let hh = textureLoad(input, vec2u(2u * id.x + 1u, 2u * id.y + 1u), 0).rgb;

    let average = correct_gamma(0.25 * (
        invert_gamma(ll) +
        invert_gamma(lh) +
        invert_gamma(hl) +
        invert_gamma(hh)
    ));

    textureStore(output, id.xy, vec4f(average, 1.0));
}
