@group(0) @binding(0)
var<storage, read> kernel: array<f32>;

@group(0) @binding(1)
var input: texture_2d<f32>;

@group(0) @binding(2)
var output: texture_storage_2d<bgra8unorm, write>;

var<push_constant> kernel_size: u32;

@compute
@workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3u) {
    let sample = textureLoad(input, id.xy, 0);
    textureStore(output, id.xy, sample);
}
