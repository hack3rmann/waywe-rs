// TODO(ArnoDarkrose): add image cropping and rotating
// TODO(ArnoDarkrose): i can parallelize mipmap decompression

use image::ImageBuffer;
use std::ffi::CString;
use std::io::Read;
use tracing::{debug, instrument};
use transmute_extra::{read_into_uninit, transmute_vec};

pub mod enums;
pub mod stages;

pub use stages::*;

use enums::*;

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct HeaderMeta {
    pub format: TexFormat,
    pub flags: TexFlags,
    pub texture_width: i32,
    pub texture_height: i32,
    pub image_width: i32,
    pub image_height: i32,
    pub unk_int0: i32,
}

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct ImageContainerMeta {
    pub version: ImageContainerVersion,
    pub image_count: i32,
    pub image_format: Option<FreeImageFormat>,
    pub is_video_mp4: bool,
}

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct TexGifContainerMeta {
    pub version: GifContainerVersion,
    pub frame_count: i32,
}

#[derive(Default, Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Mipmap {
    width: i32,
    height: i32,
    data: Vec<u8>,
    lz4_compressed: bool,
    decompressed_bytes_count: Option<i32>,
    condition_json: Option<CString>,
    format: MipmapFormat,
}

impl Mipmap {
    #[instrument(skip_all)]
    pub fn decompress(mut self) -> Result<DecompressedMipmap, TexExtractError> {
        if self.lz4_compressed {
            debug!("decompressing lz4 compressed texture");

            decompress_lz4(&mut self.data, self.decompressed_bytes_count)?;

            self.lz4_compressed = false;
        }

        let dxt_format = match self.format {
            MipmapFormat::CompressedDxt1 => dxt::DxtFormat::Dxt1,
            MipmapFormat::CompressedDxt3 => dxt::DxtFormat::Dxt3,
            MipmapFormat::CompressedDxt5 => dxt::DxtFormat::Dxt5,
            _ => return Ok(DecompressedMipmap::from(self)),
        };

        debug!("decompressing dxt texture: {dxt_format:?}");

        let data = transmute_vec(self.data)?;
        let data =
            dxt::decompress_image(self.width as usize, self.height as usize, &data, dxt_format);

        let format = MipmapFormat::Rgba8888;

        Ok(DecompressedMipmap {
            width: self.width as u32,
            height: self.height as u32,
            data: MipmapData::Rgba8888(
                ImageBuffer::from_raw(
                    self.width as u32,
                    self.height as u32,
                    transmute_extra::transmute_to_bytes_vec(data).unwrap(),
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

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct DecompressedMipmap {
    pub width: u32,
    pub height: u32,
    pub data: MipmapData,
    pub condition_json: Option<CString>,
    pub format: MipmapFormat,
}

impl From<Mipmap> for DecompressedMipmap {
    /// This function will panic if the passed mipmap is compressed
    fn from(value: Mipmap) -> Self {
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

fn decompress_lz4(
    data: &mut Vec<u8>,
    decompressed_bytes_count: Option<i32>,
) -> Result<(), TexExtractError> {
    let res = lz4::block::decompress(data.as_slice(), decompressed_bytes_count).unwrap();

    *data = res;

    Ok(())
}

#[derive(Default, Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct TexImage {
    mipmaps: Vec<Mipmap>,
}

impl TexImage {
    pub fn decompress(self) -> Result<DecompressedTexImage, TexExtractError> {
        let mipmaps = self
            .mipmaps
            .into_iter()
            .map(|mipmap| mipmap.decompress())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(DecompressedTexImage { mipmaps })
    }
}

#[derive(Default, Debug, Clone, Hash, Eq, PartialEq)]
pub struct DecompressedTexImage {
    pub mipmaps: Vec<DecompressedMipmap>,
}

#[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
pub struct TexGifFrame {
    image_id: i32,
    frame_time: f32,
    x: f32,
    y: f32,
    width: f32,
    width_y: f32,
    height_x: f32,
    height: f32,
}

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
        let mut reader = reader.read_gif_container()?.read_gif_frames()?;

        Ok(TexExtractData::Gif {
            frames: decompressed_images,
            frames_meta: reader
                .gif_frames()
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
    // #[ignore]
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
    // #[ignore]
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
    fn test_gif() {
        let mut src = BufReader::new(File::open("kurukuru.tex").unwrap());
        let mut reader = TexReader::new(&mut src)
            .read_header()
            .unwrap()
            .read_image_container_meta()
            .unwrap()
            .read_images()
            .unwrap()
            .read_gif_container()
            .unwrap()
            .read_gif_frames()
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
