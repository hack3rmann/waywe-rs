# shadertoy-computer-were-made-for-cubes

Example wallpaper using [`waywe-rendering-api`](../../waywe-rendering-api) directly.

Renders a [ShaderToy](https://www.shadertoy.com/view/3l23Rh)-style fragment shader
("Computers Were Made For Cubes") with push constants for resolution and time. You implement
all GPU setup and drawing yourself.

## FFI entry point

Unlike scene wallpapers, this example does **not** use `#[derive(Scene)]`. You must
export `waywe_ffi_create_opaque_renderer` manually:

```rust
#[unsafe(no_mangle)]
pub extern "C" fn waywe_ffi_create_opaque_renderer(
    desc: &OpaqueRendererDesc,
    out_renderer: &mut MaybeUninit<OpaqueRenderer>,
) -> PanicPayload { /* ... */ }
```

See [`src/lib.rs`](src/lib.rs) for the full implementation.

## Build

Package the renderer dylib (and any files under `assets/`, if present) into a `.ww`
archive:

```shell
waywe package build --path crates/examples/shadertoy-computer-were-made-for-cubes
```

This runs `cargo build --release` and writes
`target/release/shadertoy-computer-were-made-for-cubes.ww`.

## Run

```shell
waywe start
waywe show target/release/shadertoy-computer-were-made-for-cubes.ww
```
