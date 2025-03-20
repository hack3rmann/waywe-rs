//!Functions and structs for working the `.tex` files
//!
//!A good starting point is the [`TexReader`] struct or [`extract_data`] function
//!that does all the job done itself
//!
//! # Example of extracting data via manual work with readers and storing it via png crate
//! ```rust,ignore
//! use extractor::texture::*;
//!
//! let mut fd = std::fs::File::open("asset.tex").unwrap();
//! let mut reader = TexReader::new(&mut fd)
//!     .read_header()
//!     .unwrap()
//!     .read_image_container_meta()
//!     .unwrap()
//!     .read_images()
//!     .unwrap();
//!
//! let images = reader.images().unwrap();
//! let decompressed_images = images
//!     .into_iter()
//!     .map(|image| image.decompress())
//!     .collect::<Result<Vec<_>, _>>()
//!     .unwrap();
//!
//! for (i, image) in decompressed_images.into_iter().enumerate() {
//!     for (j, mipmap) in image.mipmaps.into_iter().enumerate() {
//!         let fd = std::fs::File::create(format!("asset{i}{j}.png")).unwrap();
//!         let mut encoder = png::Encoder::new(fd, mipmap.width as u32, mipmap.height as u32);
//!         encoder.set_color(png::ColorType::Rgba);
//!
//!         let mut writer = encoder.write_header().unwrap();
//!         writer.write_image_data(mipmap.data.as_bytes()).unwrap();
//!     }
//! }
//!
//! if reader.contains_gif() {
//!     let mut reader = reader
//!         .read_gif_container_meta()
//!         .unwrap()
//!         .read_gif_frames_meta()
//!         .unwrap();
//!
//!     let frames_meta = reader.gif_frames_meta().unwrap();
//! }
//!
//! ```
//!
//! # Example of using [`extract_data`]
//! ```rust,ignore
//!
//! use extractor::texture::*;
//!
//! let mut src = std::fs::File::open("asset.tex").unwrap();
//! let data = extract_data(&mut src).unwrap();
//!
//! match data {
//!     TexExtractData::Image(data) => {
//!         for (i, image) in data.into_iter().enumerate() {
//!             for (j, mipmap) in image.mipmaps.into_iter().enumerate() {
//!                 let out = std::fs::File::create(format!("asset{i}{j}.png")).unwrap();
//!
//!                 let mut encoder = png::Encoder::new(out, mipmap.width, mipmap.height);
//!                 encoder.set_color(png::ColorType::Rgba);
//!
//!                 let mut writer = encoder.write_header().unwrap();
//!
//!                 writer.write_image_data(mipmap.data.as_bytes()).unwrap();
//!             }
//!         }
//!     }
//!     _ => {}
//! }
//! ```

#![doc = include_str!("../../file-structure-doc.md")]

// TODO(ArnoDarkrose): add image cropping and rotating
// TODO(ArnoDarkrose): i can parallelize mipmap decompression
// TODO(ArnoDarkrose): use image instead of png in tests

use image::ImageBuffer;
use std::ffi::CString;
use std::io::Read;
use tracing::{debug, instrument};
use transmute_extra::{read_into_uninit, transmute_vec};

pub mod enums;
pub mod stages;

pub use stages::*;

pub use enums::*;

/// Information about the header of the `.tex` file
#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct HeaderMeta {
    pub format: TexFormat,
    pub flags: TexFlags,
    pub texture_width: i32,
    pub texture_height: i32,
    pub image_width: i32,
    pub image_height: i32,
    /// a value of the unknown purpose
    pub unk_int0: i32,
}

/// Information about the image container (for file structure refer to the module level documentation)
#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct TexImageContainerMeta {
    pub version: ImageContainerVersion,
    pub image_count: i32,
    pub image_format: Option<FreeImageFormat>,
    pub is_video_mp4: bool,
}

/// Information about the gif container (for file stucture refer to the module level documentation)
#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct TexGifContainerMeta {
    pub version: GifContainerVersion,
    pub frame_count: i32,
}

/// Contains (possibly) compressed data and meta info for a mipmap. In order to access the data, this has to
/// be transformed into [`DecompressedTexMipmap`] by calling [`TexMipmap::decompress`]
#[derive(Default, Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct TexMipmap {
    width: i32,
    height: i32,
    /// Buffer with the image data, can be compressed
    data: Vec<u8>,
    /// Whether the image was lz4_compressed
    lz4_compressed: bool,
    /// Size of the decompressed image data. This is uzed in lz4 decompression
    decompressed_bytes_count: Option<i32>,
    condition_json: Option<CString>,
    format: MipmapFormat,
}

impl TexMipmap {
    /// Decompresses the inner data. This includes lz4 decompression (if applied) and tranformation
    /// from one of `DXT` formats to `RGBA8888` (if applied).
    ///
    /// If the picture was not compressed by either method, nothing is done
    ///
    /// This method has to be used in order to get access to the underlying data buffer
    #[instrument(skip_all)]
    pub fn decompress(mut self) -> Result<DecompressedTexMipmap, TexExtractError> {
        if self.lz4_compressed {
            debug!("decompressing lz4 compressed texture");

            decompress_lz4(&mut self.data, self.decompressed_bytes_count)?;

            self.lz4_compressed = false;
        }

        let dxt_format = match self.format {
            MipmapFormat::CompressedDxt1 => dxt::DxtFormat::Dxt1,
            MipmapFormat::CompressedDxt3 => dxt::DxtFormat::Dxt3,
            MipmapFormat::CompressedDxt5 => dxt::DxtFormat::Dxt5,
            _ => return Ok(DecompressedTexMipmap::from(self)),
        };

        debug!("decompressing dxt texture: {dxt_format:?}");

        let data = transmute_vec(self.data)?;
        let data =
            dxt::decompress_image(self.width as usize, self.height as usize, &data, dxt_format);

        let format = MipmapFormat::Rgba8888;

        Ok(DecompressedTexMipmap {
            width: self.width as u32,
            height: self.height as u32,
            data: MipmapData::Rgba8888(
                ImageBuffer::from_raw(
                    self.width as u32,
                    self.height as u32,
                    transmute_extra::transmute_to_bytes_vec(data),
                )
                .ok_or(TexExtractError::Corrupt {
                    about: "width and height of the texture read from file are to big:\
                                    unable to create sufficient buffer"
                        .to_string(),
                })?,
            ),
            condition_json: self.condition_json,
            format,
        })
    }
}

/// Contains decompressed, ready to use image data. This shoudl be constructed from [`TexMipmap`] via a call to [`TexMipmap::decompress`]
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct DecompressedTexMipmap {
    pub width: u32,
    pub height: u32,
    /// Buffer with the image pixels
    pub data: MipmapData,
    pub condition_json: Option<CString>,
    pub format: MipmapFormat,
}

impl From<TexMipmap> for DecompressedTexMipmap {
    /// This function will panic if the passed mipmap is compressed
    fn from(value: TexMipmap) -> Self {
        if value.lz4_compressed
            || matches!(
                value.format,
                MipmapFormat::CompressedDxt5
                    | MipmapFormat::CompressedDxt3
                    | MipmapFormat::CompressedDxt1
            )
        {
            panic!("trying to construct DecompressedMipmap from a compressed Mipmap")
        };

        let data = match value.format {
            MipmapFormat::Rgba8888 => MipmapData::Rgba8888(
                ImageBuffer::from_raw(value.width as u32, value.height as u32, value.data).unwrap(),
            ),
            MipmapFormat::Rg88 => MipmapData::Rg88(
                ImageBuffer::from_raw(value.width as u32, value.height as u32, value.data).unwrap(),
            ),
            MipmapFormat::R8 => MipmapData::R8(
                ImageBuffer::from_raw(value.width as u32, value.height as u32, value.data).unwrap(),
            ),
            _ => MipmapData::Raw(value.data),
        };

        Self {
            width: value.width as u32,
            height: value.height as u32,
            condition_json: value.condition_json,
            data,
            format: value.format,
        }
    }
}

/// Decompresses lz4 compressed data
fn decompress_lz4(
    data: &mut Vec<u8>,
    decompressed_bytes_count: Option<i32>,
) -> Result<(), TexExtractError> {
    let res = lz4::block::decompress(data.as_slice(), decompressed_bytes_count)?;

    *data = res;

    Ok(())
}

/// Buffer of (possibly) compressed mipmaps of a single image. In order to get access to
/// mipmaps, this must be transformed to [`DecompressedTexImage`] via a call to [`TexImage::decompress`]
#[derive(Default, Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct TexImage {
    mipmaps: Vec<TexMipmap>,
}

impl TexImage {
    /// Decompresses mipmaps contained in `self`
    pub fn decompress(self) -> Result<DecompressedTexImage, TexExtractError> {
        let mipmaps = self
            .mipmaps
            .into_iter()
            .map(|mipmap| mipmap.decompress())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(DecompressedTexImage { mipmaps })
    }
}

/// Buffer of decompressed mipmaps. This should ultimately be constructed via a call to [`TexImage::decompress`]
#[derive(Default, Debug, Clone, Hash, Eq, PartialEq)]
pub struct DecompressedTexImage {
    pub mipmaps: Vec<DecompressedTexMipmap>,
}

/// Meta info for a single gif frame
///
/// See fields docs for the definition of frame width and height
///
/// The starting x coordinate is equal to the minimum of `x` and `x + frame_width`
///
/// The starting y coordinate is equal to the minimum of `y` and `y + frame_height`
#[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
pub struct TexGifFrameMeta {
    pub image_id: i32,
    /// Time for which the frame should be shown measured by the units of 10ms
    pub frame_time: f32,
    /// The (possibly)starting horizontal coordinate of a frame
    pub x: f32,
    /// The (possibly)starting vertical coordinate of a frame
    pub y: f32,
    /// Width of a gif frame. Can be zero in which case
    /// the width of the frame should be considered equal to `height_x`
    pub width: f32,
    /// If `height` is zero, then this is equal to frame height
    pub width_y: f32,
    /// If `width` is zero, then this is equal to frame width
    pub height_x: f32,
    /// Height of a gif frame. Can be zero in which case
    /// a height of the frame should be considered equal to `width_y`
    pub height: f32,
}

/// The function used to just get all the data from the `.tex` file without extra hassle.
///
/// `src` must be have a structure of `.tex` file
pub fn extract_data(src: &mut impl Read) -> Result<TexExtractData, TexExtractError> {
    let mut reader = TexReader::new(src)
        .read_header()?
        .read_image_container_meta()?
        .read_images()?;

    let images = reader
        .images()
        .expect("this is the first acquisition of this field so it is some");

    let decompressed_images = images
        .into_iter()
        .map(|image| image.decompress())
        .collect::<Result<Vec<_>, _>>()?;

    if reader.contains_gif() {
        let mut reader = reader.read_gif_container_meta()?.read_gif_frames_meta()?;

        Ok(TexExtractData::Gif {
            frames: decompressed_images,
            frames_meta: reader
                .gif_frames_meta()
                .expect("this is the first acquisition of this field so it is some"),
        })
    } else if reader.contains_video() {
        let first_image_bytes = &decompressed_images[0].mipmaps[0].data.as_bytes();
        if first_image_bytes.len() < 12 {
            return Err(TexExtractError::Corrupt {
                about:
                    "expected magic header for the video texture but len in bytes is not sufficient"
                        .to_string(),
            });
        }

        let magic = std::str::from_utf8(&first_image_bytes[4..9])?;

        if !matches!(magic, "ftypisom" | "ftypmsnv" | "ftypmp42") {
            return Err(TexExtractError::Corrupt {
                about: "expected magic header for the video texture".to_string(),
            });
        }

        Ok(TexExtractData::Video(decompressed_images))
    } else {
        Ok(TexExtractData::Image(decompressed_images))
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use std::io::{BufReader, BufWriter};

    use super::*;

    #[test]
    #[ignore]
    fn test_stages_up_to_images() {
        let mut fd = std::io::BufReader::new(std::fs::File::open("futaba.tex").unwrap());

        let reader = TexReader::new(&mut fd);
        let reader = reader.read_header().unwrap();

        tracing::debug!("{reader:#?}");

        let reader = reader.read_image_container_meta().unwrap();

        tracing::debug!("{reader:#?}");

        let mut reader = reader.read_images().unwrap();

        tracing::debug!("{:#?}, {:#?}", reader.header(), reader.image_container());

        let images = reader.images();

        for image in images.unwrap() {
            let image = image.decompress().unwrap();

            for (i, mipmap) in image.mipmaps.iter().enumerate() {
                tracing::debug!(
                    "mipmap_width: {:#?}, mipmap_height: {:#?}, condition_json: {:#?}",
                    mipmap.width,
                    mipmap.height,
                    mipmap.condition_json
                );

                let fd = std::fs::File::create(format!("futaba{i}.png")).unwrap();
                let mut encoder = png::Encoder::new(fd, mipmap.width as u32, mipmap.height as u32);
                encoder.set_color(png::ColorType::Rgba);

                let mut writer = encoder.write_header().unwrap();
                writer.write_image_data(mipmap.data.as_bytes()).unwrap();
            }
        }
    }

    #[test]
    #[ignore]
    fn test_dxt_image() {
        let mut src = BufReader::new(File::open("dxt_image.tex").unwrap());
        let mut reader = TexReader::new(&mut src)
            .read_header()
            .unwrap()
            .read_image_container_meta()
            .unwrap()
            .read_images()
            .unwrap();

        for image in reader.images().unwrap() {
            let image = image.decompress().unwrap();

            for (i, mipmap) in image.mipmaps.into_iter().enumerate() {
                let fd = BufWriter::new(File::create(format!("dxt_image{i}.png")).unwrap());
                let mut encoder = png::Encoder::new(fd, mipmap.width, mipmap.height);
                encoder.set_color(png::ColorType::Rgba);

                let mut writer = encoder.write_header().unwrap();

                writer.write_image_data(mipmap.data.as_bytes()).unwrap();
            }
        }
    }

    #[test]
    #[ignore]
    fn test_gif() {
        let mut src = BufReader::new(File::open("kurukuru.tex").unwrap());
        let mut reader = TexReader::new(&mut src)
            .read_header()
            .unwrap()
            .read_image_container_meta()
            .unwrap()
            .read_images()
            .unwrap()
            .read_gif_container_meta()
            .unwrap()
            .read_gif_frames_meta()
            .unwrap();

        tracing::debug!("{:#?}", reader.image_container());
        tracing::debug!("{:#?}", reader.gif_container());

        for image in reader.images().unwrap() {
            let image = image.decompress().unwrap();

            for (i, mipmap) in image.mipmaps.into_iter().enumerate() {
                let fd = BufWriter::new(File::create(format!("gif_image_mipmap{i}.png")).unwrap());
                let mut encoder = png::Encoder::new(fd, mipmap.width, mipmap.height);
                encoder.set_color(png::ColorType::Rgba);

                let mut writer = encoder.write_header().unwrap();

                writer.write_image_data(mipmap.data.as_bytes()).unwrap();
            }
        }
    }

    #[test]
    #[ignore]
    fn test_extract_basic() {
        let mut src = BufReader::new(File::open("futaba.tex").unwrap());
        let data = extract_data(&mut src).unwrap();

        match data {
            TexExtractData::Image(data) => {
                for image in data {
                    for (i, mipmap) in image.mipmaps.into_iter().enumerate() {
                        let out = BufWriter::new(File::create(format!("futaba2_{i}.png")).unwrap());

                        let mut encoder = png::Encoder::new(out, mipmap.width, mipmap.height);
                        encoder.set_color(png::ColorType::Rgba);

                        let mut writer = encoder.write_header().unwrap();

                        writer.write_image_data(mipmap.data.as_bytes()).unwrap();
                    }
                }
            }
            _ => {}
        }
    }

    #[test]
    #[ignore]
    fn test_extract_dxt() {
        let mut src = BufReader::new(File::open("dxt_image.tex").unwrap());
        let data = extract_data(&mut src).unwrap();

        match data {
            TexExtractData::Image(data) => {
                for image in data {
                    for (i, mipmap) in image.mipmaps.into_iter().enumerate() {
                        let out =
                            BufWriter::new(File::create(format!("dxt_image2_{i}.png")).unwrap());

                        let mut encoder = png::Encoder::new(out, mipmap.width, mipmap.height);
                        encoder.set_color(png::ColorType::Rgba);

                        let mut writer = encoder.write_header().unwrap();

                        writer.write_image_data(mipmap.data.as_bytes()).unwrap();
                    }
                }
            }
            _ => {}
        }
    }
}
