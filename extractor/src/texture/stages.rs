//! Various structs that represent different stages of the input `.tex` file

use super::*;
use std::{ffi::CString, io::Result as IoResult};
use tracing::{debug, instrument};

#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
struct Reader<'a, T: Read>(&'a mut T);

impl<T: Read> Reader<'_, T> {
    fn read_int(&mut self) -> IoResult<i32> {
        let mut res: i32 = 0;

        let view = safe_transmute::transmute_one_to_bytes_mut(&mut res);
        self.0.read_exact(view)?;

        Ok(res)
    }

    fn read_bytes_exact(&mut self, buf: &mut [u8]) -> IoResult<()> {
        self.0.read_exact(buf)
    }

    fn read_float(&mut self) -> IoResult<f32> {
        let mut res: f32 = 0.0;

        let view = safe_transmute::transmute_one_to_bytes_mut(&mut res);
        self.0.read_exact(view)?;

        Ok(res)
    }

    fn read_pad_byte(&mut self) -> IoResult<()> {
        _ = self.read_byte()?;
        Ok(())
    }

    fn read_byte(&mut self) -> IoResult<u8> {
        let mut byte = [0_u8];

        self.0.read_exact(&mut byte)?;

        Ok(byte[0])
    }

    fn read_cstr(&mut self) -> Result<CString, TexExtractError> {
        let mut res = Vec::new();

        let mut character = self.read_byte()?;

        // until reached nul-terminator
        while character != 0 {
            res.push(character);

            character = self.read_byte()?;
        }

        // Safety: need to push nul-terminator
        res.push(0);

        // Safety: vec has exactly one nul byte at the end
        Ok(unsafe { CString::from_vec_with_nul_unchecked(res) })
    }
}

/// Entry point for `.tex` files decompression
#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct TexReader<'a, T: Read> {
    reader: Reader<'a, T>,
}

impl<'a, T: Read> TexReader<'a, T> {
    /// `src` has to have a structure of `.tex` file
    pub fn new(src: &'a mut T) -> Self {
        Self {
            reader: Reader(src),
        }
    }

    /// Reads a header from the file
    #[instrument(skip_all)]
    pub fn read_header(mut self) -> Result<TexReaderWithHeader<'a, T>, TexExtractError> {
        let mut magic_buf = [0_u8; 8];

        self.reader.read_bytes_exact(&mut magic_buf)?;
        self.reader.read_pad_byte()?;
        let magic = std::str::from_utf8(&magic_buf)?;

        if magic != "TEXV0005" {
            return Err(TexExtractError::UnknownMagic {
                magic: magic.to_owned(),
            });
        }

        self.reader.read_bytes_exact(&mut magic_buf)?;
        self.reader.read_pad_byte()?;
        let magic = std::str::from_utf8(&magic_buf)?;

        if magic != "TEXI0001" {
            return Err(TexExtractError::UnknownMagic {
                magic: magic.to_owned(),
            });
        }

        let format = TexFormat::try_from(self.reader.read_int()?)?;

        let flags =
            u8::try_from(self.reader.read_int()?).map_err(|_| TexExtractError::InvalidTexFlags)?;
        let flags = TexFlags::from_bits(flags).ok_or(TexExtractError::InvalidTexFlags)?;

        debug!(?format, ?flags, "extracting image");

        let texture_width = self.reader.read_int()?;
        let texture_height = self.reader.read_int()?;
        let image_width = self.reader.read_int()?;
        let image_height = self.reader.read_int()?;
        let unk_int0 = self.reader.read_int()?;

        let header = HeaderMeta {
            format,
            flags,
            texture_width,
            texture_height,
            image_width,
            image_height,
            unk_int0,
        };

        Ok(TexReaderWithHeader {
            reader: self.reader,
            header,
        })
    }
}

/// `TexReader` that represents the state when only header was read from the document
#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct TexReaderWithHeader<'a, T: Read> {
    reader: Reader<'a, T>,
    header: HeaderMeta,
}

impl<'a, T: Read> TexReaderWithHeader<'a, T> {
    /// Reads the meta info for image_container
    #[instrument(skip_all)]
    pub fn read_image_container_meta(
        mut self,
    ) -> Result<TexReaderWithImageContainer<'a, T>, TexExtractError> {
        let mut magic_buf = [0_u8; 8];

        self.reader.read_bytes_exact(&mut magic_buf)?;
        self.reader.read_pad_byte()?;

        let magic = std::str::from_utf8(&magic_buf)?;

        let mut version =
            magic
                .parse::<ImageContainerVersion>()
                .map_err(|_| TexExtractError::UnknownMagic {
                    magic: magic.to_owned(),
                })?;

        let image_count = self.reader.read_int()?;

        let mut image_format = None;

        if matches!(
            version,
            ImageContainerVersion::Texb0004 | ImageContainerVersion::Texb0003
        ) {
            let fif = self.reader.read_int()?;
            image_format = Some(FreeImageFormat::from(fif));
        }

        let mut is_video_mp4 = false;

        if version == ImageContainerVersion::Texb0004 {
            is_video_mp4 = self.reader.read_int()? != 0;

            if image_format.expect("the version is 4, so the image format is present")
                == FreeImageFormat::Unknown
                && is_video_mp4
            {
                image_format = Some(FreeImageFormat::Mp4)
            }

            if !is_video_mp4 {
                version = ImageContainerVersion::Texb0003;
            }
        }

        let image_container = TexImageContainerMeta {
            version,
            image_count,
            image_format,
            is_video_mp4,
        };

        debug!(?image_container);

        Ok(TexReaderWithImageContainer {
            reader: self.reader,
            header: self.header,
            image_container,
        })
    }
}

/// TexReader that represents the state when image_container was read from the document
#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct TexReaderWithImageContainer<'a, T: Read> {
    reader: Reader<'a, T>,
    header: HeaderMeta,
    image_container: TexImageContainerMeta,
}

impl<'a, T: Read> TexReaderWithImageContainer<'a, T> {
    /// Reads single mipmap from the file
    fn read_mipmap(&mut self) -> Result<TexMipmap, TexExtractError> {
        let format =
            MipmapFormat::from_image_and_tex(self.image_container.image_format, self.header.format);

        let mut condition_json = None;

        if self.image_container.version == ImageContainerVersion::Texb0004 {
            let param1 = self.reader.read_int()?;
            if param1 != 1 {
                panic!("param1 is not 1, the library is likely outdated")
            }

            let param2 = self.reader.read_int()?;
            if param2 != 2 {
                panic!("param1 is not 2, the library is likely outdated")
            }

            condition_json = Some(self.reader.read_cstr()?);

            let param3 = self.reader.read_int()?;
            if param3 != 1 {
                panic!("param3 is not 1, the library is likely outdated")
            }
        }

        let width = self.reader.read_int()?;
        let height = self.reader.read_int()?;

        let (lz4_compressed, decompressed_bytes_count) =
            if self.image_container.version == ImageContainerVersion::Texb0001 {
                (false, None)
            } else {
                (self.reader.read_int()? == 1, Some(self.reader.read_int()?))
            };

        let byte_count = self.reader.read_int()?;

        let mut data = vec![0; byte_count as usize];
        self.reader.0.read_exact(&mut data)?;

        Ok(TexMipmap {
            width,
            height,
            data,
            lz4_compressed,
            decompressed_bytes_count,
            condition_json,
            format,
        })
    }

    /// Reads images from the file
    pub fn read_images(mut self) -> Result<TexReaderWithImages<'a, T>, TexExtractError> {
        let mut images = Vec::with_capacity(self.image_container.image_count as usize);

        for _ in 0..self.image_container.image_count {
            let mipmap_count = self.reader.read_int()?;

            let mut mipmaps = Vec::with_capacity(mipmap_count as usize);

            for _ in 0..mipmap_count {
                mipmaps.push(self.read_mipmap()?);
            }

            images.push(TexImage { mipmaps });
        }

        Ok(TexReaderWithImages {
            reader: self.reader,
            header: self.header,
            image_container: self.image_container,
            images: Some(images),
        })
    }
}

/// `TexReader` that represents the state when images were read from the file
#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct TexReaderWithImages<'a, T: Read> {
    reader: Reader<'a, T>,
    header: HeaderMeta,
    image_container: TexImageContainerMeta,
    images: Option<Vec<TexImage>>,
}

impl<'a, T: Read> TexReaderWithImages<'a, T> {
    /// Reads gif_container meta info from the file
    pub fn read_gif_container_meta(
        mut self,
    ) -> Result<TexReaderWithGifContainerMeta<'a, T>, TexExtractError> {
        let mut magic_buf = [0_u8; 8];

        self.reader.read_bytes_exact(&mut magic_buf)?;
        self.reader.read_pad_byte()?;

        let magic = std::str::from_utf8(&magic_buf)?;

        let version =
            magic
                .parse::<GifContainerVersion>()
                .map_err(|_| TexExtractError::UnknownMagic {
                    magic: magic.to_owned(),
                })?;

        let frame_count = self.reader.read_int()?;

        let meta = TexGifContainerMeta {
            version,
            frame_count,
        };

        debug!(?meta);

        Ok(TexReaderWithGifContainerMeta {
            reader: self.reader,
            header: self.header,
            image_container: self.image_container,
            images: self.images,
            gif_container: meta,
        })
    }
}

/// `TexReader` that represents the state when gif_container_meta_was read from the file
#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct TexReaderWithGifContainerMeta<'a, T: Read> {
    reader: Reader<'a, T>,
    header: HeaderMeta,
    image_container: TexImageContainerMeta,
    images: Option<Vec<TexImage>>,
    gif_container: TexGifContainerMeta,
}

impl<'a, T: Read> TexReaderWithGifContainerMeta<'a, T> {
    /// Reads meta info for a single gif frame
    fn read_frame_meta(&mut self) -> Result<TexGifFrameMeta, TexExtractError> {
        let image_id = self.reader.read_int()?;
        let frame_time = self.reader.read_float()?;
        let x;
        let y;
        let width;
        let width_y;
        let height_x;
        let height;

        if self.gif_container.version == GifContainerVersion::Texs0001 {
            x = self.reader.read_int()? as f32;
            y = self.reader.read_int()? as f32;
            width = self.reader.read_int()? as f32;
            width_y = self.reader.read_int()? as f32;
            height_x = self.reader.read_int()? as f32;
            height = self.reader.read_int()? as f32;
        } else {
            x = self.reader.read_float()?;
            y = self.reader.read_float()?;
            width = self.reader.read_float()?;
            width_y = self.reader.read_float()?;
            height_x = self.reader.read_float()?;
            height = self.reader.read_float()?;
        }

        Ok(TexGifFrameMeta {
            image_id,
            frame_time,
            x,
            y,
            width,
            width_y,
            height_x,
            height,
        })
    }

    /// Reads meta info for gif frames
    pub fn read_gif_frames_meta(
        mut self,
    ) -> Result<TexReaderWithGifFramesMeta<'a, T>, TexExtractError> {
        let (gif_width, gif_height) = if self.gif_container.version == GifContainerVersion::Texs0003
        {
            (self.reader.read_int()?, self.reader.read_int()?)
        } else {
            (0, 0)
        };

        let mut frames_meta = Vec::with_capacity(self.gif_container.frame_count as usize);

        for _ in 0..self.gif_container.frame_count {
            frames_meta.push(self.read_frame_meta()?);
        }

        debug!(?frames_meta);

        Ok(TexReaderWithGifFramesMeta {
            reader: self.reader,
            header: self.header,
            image_container: self.image_container,
            images: self.images,
            gif_container: self.gif_container,
            gif_frames: Some(frames_meta),
            gif_width,
            gif_height,
        })
    }
}

/// Reader that represents the state when all the file was read
#[derive(Debug, PartialEq, PartialOrd)]
pub struct TexReaderWithGifFramesMeta<'a, T: Read> {
    reader: Reader<'a, T>,
    header: HeaderMeta,
    image_container: TexImageContainerMeta,
    images: Option<Vec<TexImage>>,
    gif_container: TexGifContainerMeta,
    gif_frames: Option<Vec<TexGifFrameMeta>>,
    // These two fiedlds make sense only for the third version of gif container
    gif_width: i32,
    gif_height: i32,
}

impl<T: Read> TexReaderWithGifFramesMeta<'_, T> {
    /// Returns gif_frames_meta
    ///
    /// Returns `Some` if this is the first acquisition of the data and `None` otherwise
    pub fn take_gif_frames_meta(&mut self) -> Option<Vec<TexGifFrameMeta>> {
        self.gif_frames.take()
    }
}

macro_rules! impl_from_header {
    ($($Struct:ident),* $(,)?) => {
        $(
            impl<'a, T: Read> $Struct<'a, T> {
                pub fn header(&self) -> HeaderMeta {
                    self.header
                }

                /// Returns `true` if this texture contains a gif
                pub fn contains_gif(&self) -> bool {
                    self.header.flags.contains(TexFlags::IS_GIF)
                }

                /// Returns `true` if this texture contains a video
                pub fn contains_video(&self) -> bool {
                    self.header.flags.contains(TexFlags::IS_VIDEO_TEXTURE)
                }

                pub fn flags(&self) -> TexFlags {
                    self.header.flags
                }
            }
        )*
    };
}

macro_rules! impl_from_image_cotainer {
    ($($Struct:ident),* $(,)?) => {
        $(
            impl<'a, T: Read> $Struct<'a, T> {
                pub fn image_container(&self) -> TexImageContainerMeta {
                    self.image_container
                }
            }
        )*
    };
}

macro_rules! impl_from_images {
    ($($Struct:ident),* $(,)?) => {
        $(
            impl<'a, T: Read> $Struct<'a, T> {
                /// Returns images
                ///
                /// Returns `Some` if this is the first acquisition of the data and `None` otherwise
                pub fn take_images(&mut self) -> Option<Vec<TexImage>> {
                    self.images.take()
                }
            }
        )*
    };
}

macro_rules! impl_from_gif_container {
    ($($Struct:ident),* $(,)?) => {
        $(
            impl<'a, T: Read> $Struct<'a, T> {
                pub fn gif_container(&self) -> TexGifContainerMeta {
                    self.gif_container
                }
            }
        )*
    };
}

impl_from_header! {
    TexReaderWithHeader,
    TexReaderWithImageContainer,
    TexReaderWithImages,
    TexReaderWithGifContainerMeta,
    TexReaderWithGifFramesMeta,
}

impl_from_image_cotainer! {
    TexReaderWithImageContainer,
    TexReaderWithImages,
    TexReaderWithGifContainerMeta,
    TexReaderWithGifFramesMeta,
}

impl_from_images! {
    TexReaderWithImages,
    TexReaderWithGifContainerMeta,
    TexReaderWithGifFramesMeta,
}

impl_from_gif_container! {
    TexReaderWithGifContainerMeta,
    TexReaderWithGifFramesMeta,
}
