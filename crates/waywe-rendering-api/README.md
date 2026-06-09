# waywe-rendering-api

**Unstable and experimental.** The ABI and API may change without notice.

C ABI for custom `waywe` wallpaper renderers. You implement rendering yourself — this
crate only provides the interface between your code and the daemon.

The daemon loads scene wallpapers as dynamic libraries (`.so`). Each library owns its
own `wgpu` device, receives a shared Vulkan render target from the daemon, draws into
it, and the daemon copies the result to the Wayland wallpaper surface each frame.

## Architecture

```text
waywe-daemon                              renderer (.so)
     │                                           │
     │  1. Create exportable VkImage             │
     │  2. Pass FD + FfiTextureDescriptor  ────► │ import_fd_as_texture()
     │                                           │ your render() implementation
     │  ◄──────── copy_texture_to_texture ────── │
     │  3. Present to Wayland surface            │
```

Surface sharing uses `VK_KHR_external_memory_fd` with `OPAQUE_FD` handles. The daemon
and your renderer each have a separate `wgpu::Device` on the same physical GPU; the
import side must recreate an identical `VkImage` (see `import_fd_as_texture`).

On monitor resize the daemon calls `set_surface` again with a new FD and descriptor.

## Core types

| Type | Role |
|------|------|
| [`Renderer`](src/api.rs) | Trait you implement: `render()` draws a frame, `set_surface()` receives the target |
| [`OpaqueRenderer`](src/api.rs) | Type-erased, FFI-safe wrapper around any `Renderer` |
| [`RenderSurfaceFd`](src/api.rs) | File descriptor + [`FfiTextureDescriptor`](src/lib.rs) for the render target |
| [`WallpaperConfig`](https://docs.rs/waywe-runtime) | Monitor size and surface format, passed in at creation |
| [`FrameInfo`](https://docs.rs/waywe-runtime) | Returned from `render()`; hints target frame duration to the daemon |

## FFI entry point

Every renderer library must export this symbol manually:

```rust
#[unsafe(no_mangle)]
pub extern "C" fn waywe_ffi_create_opaque_renderer(
    desc: &OpaqueRendererDesc,
    out_renderer: &mut MaybeUninit<OpaqueRenderer>,
) -> PanicPayload
```

The symbol name is [`CREATE_OPAQUE_RENDERER_NAME`](src/api.rs) (`"waywe_ffi_create_opaque_renderer"`).

`OpaqueRendererDesc` currently contains only `config: WallpaperConfig`.

Panics inside your library are caught and returned as `PanicPayload`; the daemon
propagates them back into Rust with `propagate_if_any()`.

If you use [`waywe-scene`](../waywe-scene), `#[derive(Scene)]` generates this function
for you. For direct API usage you must write it yourself — see the
[shadertoy example](../examples/shadertoy-computer-were-made-for-cubes).

## Creating a renderer

### 1. Set up the crate

```toml
[lib]
crate-type = ["dylib"]

[dependencies]
waywe-rendering-api.workspace = true
waywe-runtime.workspace = true
wgpu.workspace = true
pollster.workspace = true
```

### 2. Implement `Renderer`

You are responsible for all GPU setup, pipeline creation, and drawing:

```rust
use waywe_rendering_api::{
    api::{OpaqueRenderer, OpaqueRendererDesc, RenderSurfaceFd, Renderer},
    ffi::PanicPayload,
    import_fd_as_texture,
};
use waywe_runtime::frame::FrameInfo;

struct MyRenderer { /* device, pipelines, surface, ... */ }

impl Renderer for MyRenderer {
    fn render(&mut self) -> FrameInfo {
        // Record draw commands into self.surface, submit, wait
        FrameInfo::new_60_fps()
    }

    fn set_surface(&mut self, surface: RenderSurfaceFd) {
        let texture = unsafe {
            import_fd_as_texture(&self.device, &self.adapter, surface.fd, surface.desc)
        };
        // Store texture, rebuild pipelines if format/size changed
        self.surface = Some(texture);
    }
}
```

### 3. Export the FFI entry point

```rust
use std::{mem::MaybeUninit, panic};

fn create(desc: &OpaqueRendererDesc) -> OpaqueRenderer {
    OpaqueRenderer::new(MyRenderer::new(desc.config))
}

#[unsafe(no_mangle)]
pub extern "C" fn waywe_ffi_create_opaque_renderer(
    desc: &OpaqueRendererDesc,
    out_renderer: &mut MaybeUninit<OpaqueRenderer>,
) -> PanicPayload {
    let result = panic::catch_unwind(|| create(desc));
    PanicPayload::map(result, |r| out_renderer.write(r))
}
```

### 4. Build and load

```shell
cargo build --release -p my-renderer
waywe show target/release/libmy_renderer.so
```

The CLI detects executable files (`.so`) and sends a `SetScene` command to the daemon.

## Using waywe-scene

For ECS-based scenes with images, video, meshes, and more, use [`waywe-scene`](../waywe-scene).
Put `#[derive(Scene)]` on your `WallpaperBuilder` type — the FFI entry point is
generated automatically. See the [waywe-scene README](../waywe-scene/README.md).

## Low-level Vulkan helpers

| Function | Description |
|----------|-------------|
| `import_fd_as_texture` | Import a daemon-exported FD as a `wgpu::Texture` |
| `texture_export_fd` | Export a texture's backing memory as an `OwnedFd` |
| `DeviceExt::export_fd` | Convenience trait on `wgpu::Device` |

`FfiTextureDescriptor` bridges `wgpu` and `ash`/`vk` types for cross-process sharing.

## Requirements

Your renderer's `wgpu::Device` must request the same Vulkan extensions the daemon uses:

- `VK_KHR_external_memory_fd`
- `VK_EXT_image_drm_format_modifier` (daemon side)

Both sides must use the Vulkan backend (`wgpu::Backends::VULKAN`).

## Examples

| Crate | FFI | Description |
|-------|-----|-------------|
| [`shadertoy-computer-were-made-for-cubes`](../examples/shadertoy-computer-were-made-for-cubes) | Manual | Custom renderer with GLSL shaders — you write `waywe_ffi_create_opaque_renderer` yourself |
| [`waywe-test-scene`](../examples/waywe-test-scene) | `#[derive(Scene)]` | ECS scene — FFI generated by the macro |

Build an example:

```shell
cargo build --release -p shadertoy-computer-were-made-for-cubes
waywe show target/release/libshadertoy_computer_were_made_for_cubes.so
```
