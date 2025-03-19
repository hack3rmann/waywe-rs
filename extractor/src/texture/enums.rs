use std::io;

use image::EncodableLayout;

use super::{DecompressedTexImage, TexGifFrame};

#[derive(Debug, thiserror::Error)]
pub enum TexExtractError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("encountered unkown magic in the file: {magic}")]
    UnknownMagic { magic: String },

    #[error("unknown tex_format in the file")]
    UnknownTexFormat,

    #[error("invalid tex_flags in the file")]
    InvalidTexFlags,

    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),

    #[error(transparent)]
    Transmute(#[from] transmute_extra::TransmuteVecError),

    #[error(transparent)]
    FromVecWithNul(#[from] std::ffi::FromVecWithNulError),

    #[error("corrupt data in the file: {about}")]
    Corrupt { about: String },
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
        const NONE = 0b0000_0000;
        const NO_INTERPOLATION = 0b0000_0001;
        const CLAMP_UVS = 0b0000_0010;
        const IS_GIF = 0b0000_0100;
        const UNK3 = 0b0000_1000;
        const UNK4 = 0b0001_0000;
        const IS_VIDEO_TEXTURE = 0b0010_0000;
        const UNK6 = 0b0100_0000;
        const UNK7 = 0b1000_0000;
    }
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

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum MipmapData {
    R8(image::GrayImage),
    Rg88(image::GrayAlphaImage),
    Rgba8888(image::RgbaImage),
    Raw(Vec<u8>),
}

impl MipmapData {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            MipmapData::R8(image) => image.as_bytes(),
            MipmapData::Rg88(image) => image.as_bytes(),
            MipmapData::Rgba8888(image) => image.as_bytes(),
            MipmapData::Raw(image) => image.as_bytes(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TexExtractData {
    Video(Vec<DecompressedTexImage>),
    Gif {
        frames: Vec<DecompressedTexImage>,
        frames_meta: Vec<TexGifFrame>,
    },
    Image(Vec<DecompressedTexImage>),
}
