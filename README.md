# Blazingly Fast Video Wallpapers in Rust

## Highly efficient wallpaper software with no overhead

- `waywe` stands for '**Way**land **W**allpaper **E**ngine'

### Warning

- This software is still in development, some major features may be unimplemented.
- Special warning for the ones who concerned about unsafe code in Rust: this project contains a
  lot of it and built upon it. Of course, to speed up a lot of processing code.

https://github.com/user-attachments/assets/e3ab05c0-3cb4-4fb3-911b-852020370cc8

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
hyprctl dispatch exec waywe-daemon
```

Then use the `waywe` cli tool to control deamon's behavior:

```shell
waywe video path/to/your/video.mp4
```

## Future directions

- Using [Wallpaper Engine](https://www.wallpaperengine.io/en) assets for the future use with `waywe`.
