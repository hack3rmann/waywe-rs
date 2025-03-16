// TODO(ArnoDarkrose): maybe make every new reader a deref into the previous step
// to avoid multiple similar methods and fields
// TODO(ArnoDarkrose): make transmute_vec return Result instead of Option (?)
// TODO(ArnoDarkrose): make decompressed bytes count an Option (currently it is zero if we don't need it)
// TODO(ArnoDarkrose): add image cropping and rotating and gifs

use std::io::{self, Read};
use transmute_extra::transmute_vec;

pub mod stages;

pub use stages::*;

#[derive(Debug, thiserror::Error)]
pub enum TexExtractError {
    #[error(transparent)]
    Io(#[from] io::Error),

    // TODO(ArnoDarkrose): add String that shows that magic
    #[error("encountered unkown magic in the file")]
    UnknownMagic,

    #[error("unknown tex_format in the file")]
    UnknownTexFormat,

    #[error("invalid tex_flags in the file")]
    InvalidTexFlags,

    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),

    #[error("failed to transmute Vec<u8> to Vec<u32>, input data is likely corrupt")]
    TransmuteError,

    #[error(transparent)]
    IntoString(#[from] std::ffi::IntoStringError),
}

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum TexFormat {
    #[default]
    Rgba8888 = 0,
    Dxt5 = 4,
    Dxt3 = 6,
    Dxt1 = 7,
    Rg88 = 8,
    R8 = 9,
}

impl TryFrom<i32> for TexFormat {
    type Error = TexExtractError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Rgba8888,
            4 => Self::Dxt5,
            6 => Self::Dxt3,
            7 => Self::Dxt1,
            8 => Self::Rg88,
            9 => Self::R8,
            _ => return Err(TexExtractError::UnknownTexFormat),
        })
    }
}

bitflags::bitflags! {
    #[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
    pub struct TexFlags: u8 {
        const None = 0b0000_0000;
        const NoInterpolation = 0b0000_0001;
        const ClampUVs = 0b0000_0010;
        const IsGif = 0b0000_0100;
        const Unk3 = 0b0000_1000;
        const Unk4 = 0b0001_0000;
        const IsVideoTexture = 0b0010_0000;
        const Unk6 = 0b0100_0000;
        const Unk7 = 0b1000_0000;
    }
}

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
pub enum ImageContainerVersion {
    #[default]
    Texb0004,
    Texb0003,
    Texb0002,
    Texb0001,
}

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum FreeImageFormat {
    #[default]
    Unknow = -1,
    Bmp = 0,
    Ico = 1,
    Jpeg = 2,
    Jng = 3,
    Koala = 4,
    // TODO(ArnoDarkrose): For some reason this has the same number as
    // Lbm in the original library
    Iff = 5,
    Mng = 6,
    Pbm = 7,
    PbmRaw = 8,
    Pcd = 9,
    Pcx = 10,
    Pgm = 11,
    PgmRaw = 12,
    Png = 13,
    Ppm = 14,
    PpmRaw = 15,
    Ras = 16,
    Targa = 17,
    Tiff = 18,
    Wbmp = 19,
    Psd = 20,
    Cut = 21,
    Xbm = 22,
    Xpm = 23,
    Dds = 24,
    Gif = 25,
    Hdr = 26,
    Faxg3 = 27,
    Sgi = 28,
    Exr = 29,
    J2k = 30,
    Jp2 = 31,
    Pfm = 32,
    Pict = 33,
    Raw = 34,
    Mp4 = 35,
    Lbm,
}

impl From<i32> for FreeImageFormat {
    fn from(value: i32) -> Self {
        match value {
            0 => FreeImageFormat::Bmp,
            1 => FreeImageFormat::Ico,
            2 => FreeImageFormat::Jpeg,
            3 => FreeImageFormat::Jng,
            4 => FreeImageFormat::Koala,
            5 => FreeImageFormat::Iff,
            6 => FreeImageFormat::Mng,
            7 => FreeImageFormat::Pbm,
            8 => FreeImageFormat::PbmRaw,
            9 => FreeImageFormat::Pcd,
            10 => FreeImageFormat::Pcx,
            11 => FreeImageFormat::Pgm,
            12 => FreeImageFormat::PgmRaw,
            13 => FreeImageFormat::Png,
            14 => FreeImageFormat::Ppm,
            15 => FreeImageFormat::PpmRaw,
            16 => FreeImageFormat::Ras,
            17 => FreeImageFormat::Targa,
            18 => FreeImageFormat::Tiff,
            19 => FreeImageFormat::Wbmp,
            20 => FreeImageFormat::Psd,
            21 => FreeImageFormat::Cut,
            22 => FreeImageFormat::Xbm,
            23 => FreeImageFormat::Xpm,
            24 => FreeImageFormat::Dds,
            25 => FreeImageFormat::Gif,
            26 => FreeImageFormat::Hdr,
            27 => FreeImageFormat::Faxg3,
            28 => FreeImageFormat::Sgi,
            29 => FreeImageFormat::Exr,
            30 => FreeImageFormat::J2k,
            31 => FreeImageFormat::Jp2,
            32 => FreeImageFormat::Pfm,
            33 => FreeImageFormat::Pict,
            34 => FreeImageFormat::Raw,
            35 => FreeImageFormat::Mp4,
            _ => FreeImageFormat::Unknow,
        }
    }
}

#[derive(Default, Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum MipmapFormat {
    #[default]
    Invalid = 0,
    Rgba8888 = 1,
    R8 = 2,
    Rg88 = 3,
    CompressedDxt5,
    CompressedDxt3,
    CompressedDxt1,
    VideoMp4,
    ImageBmp = 1000,
    ImageIco,
    ImageJpeg,
    ImageJng,
    ImageKoala,
    ImageLbm,
    ImageIff,
    ImageMng,
    ImagePbm,
    ImagePbmRaw,
    ImagePcd,
    ImagePcx,
    ImagePgm,
    ImagePgmRaw,
    ImagePng,
    ImagePpm,
    ImagePpmRaw,
    ImageRas,
    ImageTarga,
    ImageTiff,
    ImageWbmp,
    ImagePsd,
    ImageCut,
    ImageXbm,
    ImageXpm,
    ImageDds,
    ImageGif,
    ImageHdr,
    ImageFaxg3,
    ImageSgi,
    ImageExr,
    ImageJ2k,
    ImageJp2,
    ImagePfm,
    ImagePict,
    ImageRaw,
}

impl MipmapFormat {
    pub fn from_image_and_tex(
        image_format: Option<FreeImageFormat>,
        tex_format: TexFormat,
    ) -> Self {
        if let Some(format) = image_format {
            if format != FreeImageFormat::Unknow {
                return Self::from(format);
            }
        }

        Self::from(tex_format)
    }
}

impl From<FreeImageFormat> for MipmapFormat {
    fn from(value: FreeImageFormat) -> Self {
        match value {
            FreeImageFormat::Unknow => {
                panic!("unexpected ImageFormat::Unknown when coverting to MipmapFormat")
            }
            FreeImageFormat::Bmp => MipmapFormat::ImageBmp,
            FreeImageFormat::Ico => MipmapFormat::ImageIco,
            FreeImageFormat::Jpeg => MipmapFormat::ImageJpeg,
            FreeImageFormat::Jng => MipmapFormat::ImageJng,
            FreeImageFormat::Koala => MipmapFormat::ImageKoala,
            FreeImageFormat::Iff => MipmapFormat::ImageLbm,
            FreeImageFormat::Mng => MipmapFormat::ImageMng,
            FreeImageFormat::Pbm => MipmapFormat::ImagePbm,
            FreeImageFormat::PbmRaw => MipmapFormat::ImagePbmRaw,
            FreeImageFormat::Pcd => MipmapFormat::ImagePcd,
            FreeImageFormat::Pcx => MipmapFormat::ImagePcx,
            FreeImageFormat::Pgm => MipmapFormat::ImagePcx,
            FreeImageFormat::PgmRaw => MipmapFormat::ImagePgmRaw,
            FreeImageFormat::Png => MipmapFormat::ImagePng,
            FreeImageFormat::Ppm => MipmapFormat::ImagePpm,
            FreeImageFormat::PpmRaw => MipmapFormat::ImagePpmRaw,
            FreeImageFormat::Ras => MipmapFormat::ImageRas,
            FreeImageFormat::Targa => MipmapFormat::ImageTarga,
            FreeImageFormat::Tiff => MipmapFormat::ImageTiff,
            FreeImageFormat::Wbmp => MipmapFormat::ImageWbmp,
            FreeImageFormat::Psd => MipmapFormat::ImagePsd,
            FreeImageFormat::Cut => MipmapFormat::ImageCut,
            FreeImageFormat::Xbm => MipmapFormat::ImageXbm,
            FreeImageFormat::Xpm => MipmapFormat::ImageXpm,
            FreeImageFormat::Dds => MipmapFormat::ImageDds,
            FreeImageFormat::Gif => MipmapFormat::ImageGif,
            FreeImageFormat::Hdr => MipmapFormat::ImageHdr,
            FreeImageFormat::Faxg3 => MipmapFormat::ImageFaxg3,
            FreeImageFormat::Sgi => MipmapFormat::ImageSgi,
            FreeImageFormat::Exr => MipmapFormat::ImageExr,
            FreeImageFormat::J2k => MipmapFormat::ImageJ2k,
            FreeImageFormat::Jp2 => MipmapFormat::ImageJp2,
            FreeImageFormat::Pfm => MipmapFormat::ImagePfm,
            FreeImageFormat::Pict => MipmapFormat::ImagePict,
            FreeImageFormat::Mp4 => MipmapFormat::VideoMp4,
            FreeImageFormat::Lbm => MipmapFormat::ImageLbm,
            FreeImageFormat::Raw => MipmapFormat::ImageRaw,
        }
    }
}

impl From<TexFormat> for MipmapFormat {
    fn from(value: TexFormat) -> Self {
        match value {
            TexFormat::Rgba8888 => Self::Rgba8888,
            TexFormat::Dxt5 => Self::CompressedDxt5,
            TexFormat::Dxt3 => Self::CompressedDxt3,
            TexFormat::Dxt1 => Self::CompressedDxt1,
            TexFormat::Rg88 => Self::Rg88,
            TexFormat::R8 => Self::R8,
        }
    }
}

#[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum GifContainerVersion {
    Texs0001,
    Texs0002,
    #[default]
    Texs0003,
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
    // makes sense only for versions 2 and 3
    decompressed_bytes_count: i32,
    condition_json: Option<String>,
    format: MipmapFormat,
}

#[derive(Default, Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct DecompressedMipmap {
    width: u32,
    height: u32,
    data: Vec<u32>,
    condition_json: Option<String>,
    format: MipmapFormat,
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
                let data = transmute_vec(self.data).ok_or(TexExtractError::TransmuteError)?;
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
                let data = transmute_vec(self.data).ok_or(TexExtractError::TransmuteError)?;
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
                let data = transmute_vec(self.data).ok_or(TexExtractError::TransmuteError)?;
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
    decompressed_bytes_count: i32,
) -> Result<(), TexExtractError> {
    let res = lz4::block::decompress(data.as_slice(), Some(decompressed_bytes_count)).unwrap();

    *data = res;

    Ok(())
}

#[derive(Default, Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct TexImage {
    mipmaps: Vec<Mipmap>,
}

#[derive(Default, Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct DecompressedTexImage {
    pub mipmaps: Vec<DecompressedMipmap>,
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

/// Reads exactly `n` bytes from `src` into an initialized buffer
/// and returns it as a [`Vec`]
pub fn read_into_uninit(src: &mut impl std::io::Read, n: usize) -> std::io::Result<Vec<u8>> {
    let mut data = Box::new_uninit_slice(n);
    let ptr = data.as_mut_ptr().cast();

    // Safety
    //
    // - `ptr` is non-null and aligned as it was allocated via a call to `Box::new_uninit_slice`
    // - `ptr` is allocated within a single object via a call to `Box::new_uinit_slice`
    // - `ptr` points to n elements as this exact value was passed to `new_uninit_slice`
    // - there are no other pointers that access this memory while this slice is being operated on
    // - `Box::new_uninit_slice` wouldn't let to create an allocation that is more than isize::MAX bytes
    let dest_slice = unsafe { std::slice::from_raw_parts_mut(ptr, n) };

    src.read_exact(dest_slice)?;

    // Safety
    //
    // The contents of the slice were initialized above
    let data = unsafe { data.assume_init() };

    Ok(Vec::from(data))
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use io::{BufReader, BufWriter};

    use super::*;

    #[test]
    // #[ignore]
    fn test_stages_up_to_images() {
        let fd = io::BufReader::new(std::fs::File::open("futaba.tex").unwrap());

        let reader = TexReader::new(fd);
        let reader = reader.read_header().unwrap();

        dbg!(&reader);

        let reader = reader.read_image_container_meta().unwrap();

        dbg!(&reader);

        let mut reader = reader.read_images().unwrap();

        println!(
            "{:#?}, {:#?}",
            reader.header(),
            reader.image_container_meta()
        );

        let images = reader.images();

        for image in images.unwrap() {
            println!("here");

            let image = image.decompress().unwrap();

            for (i, mipmap) in image.mipmaps.iter().enumerate() {
                println!(
                    "mipmap_width: {:#?}, mipmap_height: {:#?}, condition_json: {:#?}",
                    mipmap.width, mipmap.height, mipmap.condition_json
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

        dbg!(&reader.image_container());
        dbg!(&reader.gif_container());

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
}
