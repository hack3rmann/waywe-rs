@group(0) @binding(0)
var input: texture_2d<f32>;

@group(0) @binding(1)
var input_sampler: sampler;

@group(0) @binding(2)
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
    let coord = vec2f(id.xy) / (vec2f(textureDimensions(output)) - vec2f(1.0));
    let sample = textureSampleLevel(input, input_sampler, coord, 0);
    textureStore(output, id.xy, sample);
}
