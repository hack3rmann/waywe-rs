# Blazingly Fast Video Wallpapers in Rust

## Highly efficient wallpaper software with no overhead

- `waywe` stands for '**Way**land **W**allpaper **E**ngine'

https://github.com/user-attachments/assets/48a8b135-bbf2-4055-8453-19292a923939

### Warning

- This software is still in development, some major features may be unimplemented.
- This project uses a lot of hardware-dependent code to make it work fast as _f_.
- Special warning for the ones who concerned about unsafe code in Rust: this project contains a
  lot of it and built upon it. Of course, to speed up a lot of processing code.

## Available Features

1. Image wallpapers in various formats.
2. Video wallpapers in .mp4 (h.264 and h.265 -encoded formats)
3. Change transition.

## Dependencies

1. Modern Linux distribution.
2. `wlroots`-based Wayland compositor (e.g. Hyprland or Sway).
3. Support for `libva` hardware acceleration.
4. Support for minor (yet unpopular) Vulkan features:
    - VK_KHR_external_memory_fd
    - VK_EXT_external_memory_dma_buf
    - VK_EXT_image_drm_format_modifier
    - vkImageDrmFormatModifierExplicitCreateInfoEXT (for some reason may be unavailable)

## Build

```shell
cargo build --release
```

## Install

After build instructions run:

```shell
sudo cp /target/release/waywe /usr/bin/waywe
sudo cp /target/release/waywe-daemon /usr/bin/waywe-daemon
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
waywe video path/to/your/video.mp4
waywe image path/to/your/picture.jpg
```

Also, you can create preview of currently running wallpaper:

```shell
waywe preview preview.png
```

For other handy commands run `waywe help`.

## Troubleshooting

This project is tested only on several machines with Intel or AMD
CPUs with integrated graphics running Fedora 42.

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

- Using [Wallpaper Engine](https://www.wallpaperengine.io/en) assets for the future use with `waywe`.
