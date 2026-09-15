# Blazingly Fast Video Wallpapers in Rust

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/hack3rmann/waywe-rs)

## Highly efficient wallpaper software with no overhead

- `waywe` stands for '**Way**land **W**allpaper **E**ngine'

https://github.com/user-attachments/assets/48a8b135-bbf2-4055-8453-19292a923939

### Warning

- This software is still in development, some major features may be unimplemented.
- This project uses a lot of hardware-dependent code to make it work fast as *F*.
- Special warning for the ones who concerned about unsafe code in Rust: this project contains a
  lot of it and built upon it. Of course, to speed up a lot of processing code.

## Available Features

1. Image wallpapers in various formats.
2. Video wallpapers in .mp4 (h.264 and h.265 -encoded formats)
3. Configurable transition animations.
4. Custom scene wallpapers packaged as `.ww` archives (shared library + assets).

## Dependencies

1. Modern Linux distribution.
2. `wlroots`-based Wayland compositor (e.g. Hyprland or Sway).
3. Support for `libva` hardware acceleration.

## Install

### Clone the repo

```shell
git clone https://github.com/hack3rmann/waywe-rs.git --depth 1
```

### Build

Build whole project

```shell
cd waywe-rs
cargo build --release --locked
```

### Install

You can find both `waywe` and `waywe-daemon` executables under `target/release` directory:

```shell
sudo cp target/release/waywe target/release/waywe-daemon /usr/bin
```

## Usage

Start the daemon:

```shell
waywe start
# or
waywe-daemon --run-in-background
```

Then use the `waywe` cli tool to control daemon's behavior:

```shell
waywe show path/to/your/video.mp4
waywe show path/to/your/picture.jpg
waywe show path/to/your/scene.ww
```

Scene wallpapers are dynamic libraries built with [`waywe-scene`](crates/waywe-scene) or
[`waywe-rendering-api`](crates/waywe-rendering-api) (experimental), then packaged with
`waywe package build` so the `.so` and any `assets/` are shipped together. See the crate
READMEs for how to author, package, and load them.

Note that it will set the same wallpaper for all currently plugged monitors.
You can also specify on which monitor to set wallpaper to with `--monitor <NAME>` key.

Also, you can create preview image of currently running wallpaper:

```shell
waywe preview preview.png
```

For other handy commands run `waywe help`.

### Scene wallpapers

With [`waywe-scene`](crates/waywe-scene), put `#[derive(Scene)]` on your wallpaper type —
the FFI entry point is generated for you. With [`waywe-rendering-api`](crates/waywe-rendering-api)
directly, you write `waywe_ffi_create_opaque_renderer` yourself and implement all rendering.

Package an example scene and set it as wallpaper:

```shell
waywe package build --path crates/examples/waywe-test-scene
waywe show target/release/waywe-test-scene.ww
```

ShaderToy example:

```shell
waywe package build --path crates/examples/shadertoy-computer-were-made-for-cubes
waywe show target/release/shadertoy-computer-were-made-for-cubes.ww
```

`waywe package build` runs `cargo build --release`, bundles `lib<crate>.so` as `wallpaper.so`, and
includes every file under the crate's `assets/` directory into a gzip-compressed tar
archive (`.tar.gz` aliased as `.ww`) under `target/<profile>/`.

## Configuration

Configuration is written in `Dhall` language. See config [docs](docs/config/README.md).

## Troubleshooting

This project is tested only on several machines with Intel or AMD
CPUs with integrated graphics running Fedora 42/43/44.

### Common issues

1. `ERROR_FORMAT_NOT_SUPPORTED`:
    - try install `ffmpeg` and `libva` libraries
    - try update/install your video drivers
2. You have both discrete and integrated graphics:
    - try `vainfo | grep Driver` - it will show the current driver name.
    - if you are on Intel, set `LIBVA_DRIVER_NAME=iHD` environment variable before you run the daemon.
    - or for AMD, set `LIBVA_DRIVER_NAME=Gallium`
    - otherwise set it accordingly with your integrated graphics driver.

## Alternatives

There are already tools with quite similar features:

- [`swww`](https://github.com/LGFae/swww) - great tool to use with picture wallpapers.
- [`swaybg`](https://github.com/swaywm/swaybg) - from the authors of `wlroots` protocol.
- [`mpvpaper`](https://github.com/GhostNaN/mpvpaper) - play videos with `mpv` directly on your wallpaper.
- [`hyprpaper`](https://github.com/hyprwm/hyprpaper) - simplest solution for Hyprland users

## Acknowledgments

Special thanks to [`swww`](https://github.com/LGFae/swww). `waywe` project is heavily inspired by `swww`.

## Future directions

- Using [Wallpaper Engine](https://www.wallpaperengine.io/en) assets with `waywe`.
