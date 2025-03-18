// TODO(ArnoDarkrose): add image cropping and rotating and gifs
// TODO(ArnoDarkrose): add tracing
// TODO(ArnoDarkrose): i can parallelize mipmap decompression

use safe_transmute::transmute_to_bytes;
use std::ffi::CString;
use std::io::{Read, Write};
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
pub struct GifContainerMeta {
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

#[derive(Default, Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct DecompressedMipmap {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u32>,
    pub condition_json: Option<CString>,
    pub format: MipmapFormat,
}

impl From<Mipmap> for DecompressedMipmap {
    /// panics if the mipmap is compressed
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

        Self {
            width: value.width as u32,
            height: value.height as u32,
            condition_json: value.condition_json,
            data: transmute_vec(value.data)
                .expect("trying to construct DecompressedMipmap from a compressed Mipmap"),
            format: value.format,
        }
    }
}

impl Mipmap {
    pub fn decompress(mut self) -> Result<DecompressedMipmap, TexExtractError> {
        if self.lz4_compressed {
            decompress_lz4(&mut self.data, self.decompressed_bytes_count)?;

            self.lz4_compressed = false;
        }

        if self.format == MipmapFormat::VideoMp4 {
            return Ok(DecompressedMipmap::from(self));
        }

        match self.format {
            MipmapFormat::CompressedDxt1 => {
                let data = transmute_vec(self.data)?;
                let data = dxt::decompress_image(
                    self.width as usize,
                    self.height as usize,
                    &data,
                    dxt::DxtFormat::Dxt1,
                );

                let format = MipmapFormat::Rgba8888;

                Ok(DecompressedMipmap {
                    width: self.width as u32,
                    height: self.height as u32,
                    data,
                    condition_json: self.condition_json,
                    format,
                })
            }
            MipmapFormat::CompressedDxt3 => {
                let data = transmute_vec(self.data)?;
                let data = dxt::decompress_image(
                    self.width as usize,
                    self.height as usize,
                    &data,
                    dxt::DxtFormat::Dxt3,
                );

                let format = MipmapFormat::Rgba8888;

                Ok(DecompressedMipmap {
                    width: self.width as u32,
                    height: self.height as u32,
                    data,
                    condition_json: self.condition_json,
                    format,
                })
            }
            MipmapFormat::CompressedDxt5 => {
                let data = transmute_vec(self.data)?;
                let data = dxt::decompress_image(
                    self.width as usize,
                    self.height as usize,
                    &data,
                    dxt::DxtFormat::Dxt5,
                );

                let format = MipmapFormat::Rgba8888;

                Ok(DecompressedMipmap {
                    width: self.width as u32,
                    height: self.height as u32,
                    data,
                    condition_json: self.condition_json,
                    format,
                })
            }
            _ => Ok(DecompressedMipmap::from(self)),
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

#[derive(Default, Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
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

pub fn extract_data(src: impl Read) -> Result<TexExtractData, TexExtractError> {
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

    if reader.header().flags.contains(TexFlags::IsGif) {
        todo!()
    } else if reader.header().flags.contains(TexFlags::IsVideoTexture) {
        let first_image_bytes = transmute_to_bytes(&decompressed_images[0].mipmaps[0].data);
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
        let fd = std::io::BufReader::new(std::fs::File::open("futaba.tex").unwrap());

        let reader = TexReader::new(fd);
        let reader = reader.read_header().unwrap();

        tracing::debug!("{reader:#?}");

        let reader = reader.read_image_container_meta().unwrap();

        tracing::debug!("{reader:#?}");

        let mut reader = reader.read_images().unwrap();

        tracing::debug!(
            "{:#?}, {:#?}",
            reader.header(),
            reader.image_container_meta()
        );

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
                writer
                    .write_image_data(safe_transmute::transmute_to_bytes(mipmap.data.as_slice()))
                    .unwrap();
            }
        }
    }

    #[test]
    // #[ignore]
    fn test_dxt_image() {
        let src = BufReader::new(File::open("dxt_image.tex").unwrap());
        let mut reader = TexReader::new(src)
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

                writer
                    .write_image_data(safe_transmute::transmute_to_bytes(mipmap.data.as_slice()))
                    .unwrap();
            }
        }
    }

    #[test]
    fn test_gif() {
        let src = BufReader::new(File::open("kurukuru.tex").unwrap());
        let mut reader = TexReader::new(src)
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

                writer
                    .write_image_data(safe_transmute::transmute_to_bytes(mipmap.data.as_slice()))
                    .unwrap();
            }
        }
    }

    #[test]
    fn test_extract_basic() {
        let src = BufReader::new(File::open("futaba.tex").unwrap());
        let data = extract_data(src).unwrap();

        match data {
            TexExtractData::Image(data) => {
                for image in data {
                    for (i, mipmap) in image.mipmaps.into_iter().enumerate() {
                        let out = BufWriter::new(File::create(format!("futaba2_{i}.png")).unwrap());

                        let mut encoder = png::Encoder::new(out, mipmap.width, mipmap.height);
                        encoder.set_color(png::ColorType::Rgba);

                        let mut writer = encoder.write_header().unwrap();

                        writer
                            .write_image_data(safe_transmute::transmute_to_bytes(
                                mipmap.data.as_slice(),
                            ))
                            .unwrap();
                    }
                }
            }
            _ => {}
        }
    }

    #[test]
    fn test_extract_dxt() {
        let src = BufReader::new(File::open("dxt_image.tex").unwrap());
        let data = extract_data(src).unwrap();

        match data {
            TexExtractData::Image(data) => {
                for image in data {
                    for (i, mipmap) in image.mipmaps.into_iter().enumerate() {
                        let out =
                            BufWriter::new(File::create(format!("dxt_image2_{i}.png")).unwrap());

                        let mut encoder = png::Encoder::new(out, mipmap.width, mipmap.height);
                        encoder.set_color(png::ColorType::Rgba);

                        let mut writer = encoder.write_header().unwrap();

                        writer
                            .write_image_data(safe_transmute::transmute_to_bytes(
                                mipmap.data.as_slice(),
                            ))
                            .unwrap();
                    }
                }
            }
            _ => {}
        }
    }
}
