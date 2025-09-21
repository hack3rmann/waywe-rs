//! Default wallpaper implementations.
//!
//! This module provides several default wallpaper implementations that can be
//! used out-of-the-box or as examples for creating custom wallpapers.
//!
//! # Available Default Wallpapers
//!
//! - [`ImageWallpaper`](image::ImageWallpaper): Display a static image as wallpaper
//! - [`VideoWallpaper`](video::VideoWallpaper): Play a video as wallpaper
//! - [`SceneTestWallpaper`](test::SceneTestWallpaper): Test wallpaper with multiple meshes and animations
//!
//! # Usage
//!
//! ```rust
//! use waywe_daemon::wallpaper::default::image::ImageWallpaper;
//! use waywe_daemon::wallpaper::scene::wallpaper::Wallpaper;
//! use std::path::PathBuf;
//!
//! // Create an image wallpaper
//! let image_wallpaper = ImageWallpaper {
//!     path: PathBuf::from("path/to/image.png"),
//! };
//!
//! // The wallpaper can then be built into a Wallpaper instance
//! // using the WallpaperBuilder trait
//! ```

pub mod image;
pub mod test;
pub mod video;
