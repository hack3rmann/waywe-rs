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

Test assets are downloaded automatically by `build.rs`.

```shell
cargo build --release -p waywe-test-scene
```

## Run

```shell
waywe start
waywe show target/release/libwaywe_test_scene.so
```
