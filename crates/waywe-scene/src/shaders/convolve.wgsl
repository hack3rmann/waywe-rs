@group(0) @binding(0)
var<storage, read> kernel: array<f32>;

@group(0) @binding(1)
var input: texture_2d<f32>;

@group(0) @binding(2)
var output: texture_storage_2d<bgra8unorm, write>;

var<push_constant> kernel_size: u32;

fn invert_gamma(sample: vec3f) -> vec3f {
    return pow(sample, vec3f(2.2));
}

fn correct_gamma(sample: vec3f) -> vec3f {
    return pow(sample, vec3f(1.0 / 2.0));
}

fn sample_kernel(id: vec2u) -> f32 {
    return kernel[id.y * kernel_size + id.x];
}

fn convolve_at(id: vec2u) -> vec3f {
    var result = vec3f(0.0);

    for (var i = 0u; i < kernel_size; i++) {
        for (var j = 0u; j < kernel_size; j++) {
            let kernel_id = kernel_size - vec2u(i, j) - vec2u(1u);
            let image_id = vec2u(vec2i(id) - vec2i(i32(i), i32(j)) + vec2i(i32(kernel_size) / 2));

            let sample = invert_gamma(textureLoad(input, image_id, 0).rgb);
            let coeff = sample_kernel(kernel_id);

            result += coeff * sample;
        }
    }

    return correct_gamma(result);
}

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3u) {
    let sample = convolve_at(id.xy);
    textureStore(output, id.xy, vec4f(sample, 1.0));
}
