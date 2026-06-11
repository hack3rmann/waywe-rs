# waywe-scene

**Unstable and experimental.** Built on top of the experimental
[`waywe-rendering-api`](../waywe-rendering-api).

ECS framework for building dynamic `waywe` wallpapers. Built on
[Bevy ECS](https://bevyengine.org/) with components and systems for images, video,
meshes, sprites, transforms, and cursor interaction.

Scene wallpapers are dynamic libraries packaged as `.ww` archives and loaded by the
daemon. Your type must have `#[derive(Scene)]` -- this generates the
`waywe_ffi_create_opaque_renderer` FFI symbol required by the rendering API. Without it
you would need to write that function manually (see [`waywe-rendering-api`](../waywe-rendering-api)).

## Architecture

Each wallpaper uses two ECS worlds that stay in sync:

```text
┌─────────────────────┐         extract          ┌─────────────────────┐
│     Main World      │  ──────────────────────► │    Render World     │
│  logic, assets,     │                          │  GPU resources,     │
│  Update systems     │                          │  draw calls         │
└─────────────────────┘                          └─────────────────────┘
         │                                                  │
         │  runs on background thread (after first frame)   │
         └──────────────── frame loop ──────────────────────┘
```

**Schedules**

| Schedule | World | Purpose |
|----------|-------|---------|
| `Startup` / `PostStartup` | Main | One-time setup |
| `Update` | Main | Per-frame logic |
| `SceneExtract` | Render | Copy main-world data into render world |
| `Render` | Render | GPU rendering (clear, draw, effects) |

## Quick start

### 1. Create a scene crate

```toml
[lib]
crate-type = ["dylib"]

[dependencies]
waywe-scene.workspace = true
bevy_ecs.workspace = true
```

### 2. Implement `WallpaperBuilder` with `#[derive(Scene)]`

`#[derive(Scene)]` is **required**. It generates the FFI entry point the daemon loads.

```rust
use waywe_scene::prelude::*;

#[derive(Default, Scene)]
pub struct MyWallpaper;

impl WallpaperBuilder for MyWallpaper {
    fn build(self, wallpaper: &mut Wallpaper) {
        wallpaper.add_plugins(DefaultPlugins);

        wallpaper
            .main
            .add_systems(Startup, setup)
            .add_systems(Update, animate);
    }
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // spawn entities, load assets ...
}

fn animate(time: Res<Time>, mut query: Query<&mut Transform>) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_rotation_z(time.elapsed.as_secs_f32());
    }
}
```

### 3. Add assets (optional)

Put runtime files (images, videos, meshes, etc.) in an `assets/` directory next to
`Cargo.toml`. They are bundled into the package and resolved at runtime from the
unpacked package root via the `WorkingDir` resource and `AssetServer`.

### 4. Package and run

```shell
waywe package build --path path/to/my-wallpaper
waywe show target/release/my-wallpaper.ww
```

`waywe package build` compiles the dylib, renames it to `wallpaper.so` inside the
archive, and includes all files from `assets/`. The output `.ww` file is written to
`target/<profile>/`.

## Core concepts

### ECS plugins

Plugins add systems and resources to one or both worlds:

```rust
use waywe_scene::{plugin::Plugin, wallpaper::Wallpaper};

struct MyPlugin;

impl Plugin for MyPlugin {
    fn build(&self, wallpaper: &mut Wallpaper) {
        wallpaper.main.add_systems(Update, my_system);
    }
}
```

**Built-in plugins** (available via `DefaultPlugins`):

| Plugin | Provides |
|--------|----------|
| `TransformPlugin` | `Transform`, `GlobalTransform` |
| `ImagePlugin` | `Image`, `ImageMaterial` |
| `VideoPlugin` | `Video`, `VideoMaterial` |
| `MeshPlugin` | `Mesh`, `Mesh3d`, `MeshMaterial` |
| `SpritePlugin` | `Sprite` |
| `MaterialPlugin` | Shared material infrastructure |
| `CursorPlugin` | `Cursor` resource (pointer position) |
| `ClearScreenPlugin` | Background clear pass |
| `AssetServerPlugin` | Async asset loading |

Add plugins with `wallpaper.add_plugins(DefaultPlugins)` or individually.

### Assets

Assets live in the main world's `Assets<T>` storage and are extracted to
`RenderAssets<T>` in the render world each frame. Use `AssetHandle<T>` for
type-safe references.

Place files under `assets/` in your crate so `waywe package build` includes them.
At runtime the daemon unpacks the `.ww` archive and sets `WorkingDir` to the package
root; load paths relative to `assets/`:

```rust
let mut path = PathBuf::from(world.resource::<WorkingDir>().0.as_str());
path.push("assets");
path.push("texture.png");
let handle = images.add(Image::from_path(path)?);
```

Or use `AssetServer::load("texture.png")` after adding `AssetServerPlugin`.
It will resolve all paths relative to the asset directory automatically.

### Frame rate

Configure via `WallpaperConfig::framerate`:

```rust
use waywe_scene::{FrameRateSetting, WallpaperConfig};
use std::time::Duration;

WallpaperConfig {
    framerate: FrameRateSetting::TargetFrameDuration(Duration::from_millis(16)),
}
```

- `TargetFrameDuration` — fixed target (e.g. 60 FPS)
- `GuessFromScene` — match the fastest video in the scene
- `NoUpdate` — static wallpaper, skip update systems

### Monitor

The `Monitor` resource provides `size: UVec2` and `surface_format`. It is updated
automatically when the daemon resizes the surface.

## Components reference

| Component | Description |
|-----------|-------------|
| `Transform` | Local position, rotation, scale |
| `GlobalTransform` | World-space transform (computed) |
| `Image` / `ImageMaterial` | Static image rendering |
| `Video` / `VideoMaterial` | Hardware-decoded video textures |
| `Mesh` / `Mesh3d` / `MeshMaterial` | Arbitrary geometry |
| `Sprite` | Billboard quad with texture |
| `Cursor` | Normalized pointer position (resource) |

## Shaders

Scene shaders use `#[derive(ShaderDescriptor)]` from `waywe-spirv-derive` with
`waywe_runtime::shaders::ShaderDescriptor`. GLSL sources live next to the Rust code
and are compiled at build time.

## FFI integration

When loaded from a `.ww` package, the daemon unpacks the archive and calls
`waywe_scene::ffi::create_opaque_renderer`, which creates a `SceneRenderer` that:

1. Initializes its own `Gpu` (separate `wgpu` device from the daemon)
2. Sets `WorkingDir` to the unpacked package root (where `assets/` lives)
3. Builds your `WallpaperBuilder` implementation
4. Imports the daemon's render surface via FD on `set_surface`
5. Runs the dual-world frame loop and returns `FrameInfo`

With `#[derive(Scene)]` you do not write any FFI code yourself. For custom renderers
that skip the ECS layer, use [`waywe-rendering-api`](../waywe-rendering-api) directly
and implement the FFI entry point manually.

## Example

See [`waywe-test-scene`](../examples/waywe-test-scene) for a full scene with multiple
meshes, images, videos, and animation systems.

```shell
waywe package build --path crates/examples/waywe-test-scene
waywe show target/release/waywe-test-scene.ww
```

## Custom renderers without ECS

If you do not need the ECS scene system, implement [`waywe_rendering_api::Renderer`](../waywe-rendering-api)
directly and write `waywe_ffi_create_opaque_renderer` yourself. See the
[shadertoy example](../examples/shadertoy-computer-were-made-for-cubes).
