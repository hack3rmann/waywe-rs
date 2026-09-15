# waywe-test-scene

Example scene wallpaper using [`waywe-scene`](../../waywe-scene).

Demonstrates multiple meshes, images, videos, and animation systems (rotation, cursor
response, entity spawning/despawning).

## Requirements

Your wallpaper type **must** have `#[derive(Scene)]`. This generates the
`waywe_ffi_create_opaque_renderer` FFI symbol the daemon expects. You do not write
that function yourself.

```rust
#[derive(Default, Scene)]
pub struct SceneTestWallpaper;

impl WallpaperBuilder for SceneTestWallpaper { /* ... */ }
```

## Build

Test assets are downloaded automatically by `build.rs` into `assets/`. Package the
dylib together with those assets into a `.ww` archive:

```shell
waywe package build --path crates/examples/waywe-test-scene
```

This runs `cargo build --release`, bundles `libwaywe_test_scene.so` as `wallpaper.so`,
and includes every file under `assets/`. The output is written to
`target/release/waywe-test-scene.ww`.

## Run

```shell
waywe start
waywe show target/release/waywe-test-scene.ww
```
