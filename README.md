# Blazingly Fast Video Wallpapers in Rust

## Highly efficient wallpaper software with no overhead

- `waywe` stands for '**Way**land **W**allpaper **E**ngine'

### Warning

- This software is still in development, some major features may be unimplemented.
- This project uses a lot of hardware-dependent code to make it work fast as *f*.
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

Start the daemon. For example, in hyprland, you run:

```shell
hyprctl dispatch exec "waywe-daemon path/to/your/default/video"
```

Then use the `waywe` cli tool to control daemon's behavior:

```shell
waywe video path/to/your/video.mp4   # For video files
waywe image path/to/your/picture.jpg # For pictires
```

## Future directions

- Using [Wallpaper Engine](https://www.wallpaperengine.io/en) assets for the future use with `waywe`.
