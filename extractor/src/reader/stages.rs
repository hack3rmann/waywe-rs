use super::*;
use std::{
    ffi::CString,
    io::{Read, Result as IoResult},
};

#[derive(Default, Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
struct Reader<T: Read>(T);

impl<T: Read> Reader<T> {
    fn read_int(&mut self) -> IoResult<i32> {
        let mut res: i32 = 0;

        let view = safe_transmute::transmute_one_to_bytes_mut(&mut res);
        self.0.read_exact(view)?;

        Ok(res)
    }

    fn read_string(&mut self, buf: &mut [u8]) -> IoResult<()> {
        self.0.read_exact(buf)
    }

    fn read_float(&mut self) -> IoResult<f32> {
        let mut res: f32 = 0.0;

        let view = safe_transmute::transmute_one_to_bytes_mut(&mut res);
        self.0.read_exact(view)?;

        Ok(res)
    }

    fn read_pad_byte(&mut self) -> IoResult<()> {
        self.read_byte()?;
        Ok(())
    }

    fn read_byte(&mut self) -> IoResult<u8> {
        let mut byte = [0_u8];

        self.0.read_exact(&mut byte)?;

        Ok(byte[0])
    }

    fn read_cstr(&mut self) -> IoResult<CString> {
        let mut res = Vec::new();

        let mut character = self.read_byte()?;

        // until reached null-terminator
        while character != 0 {
            res.push(character);

            character = self.read_byte()?;
        }
        res.push(0);

        Ok(CString::from_vec_with_nul(res).expect("null terminator is inserted manually above"))
    }
}

/// Entry point for `.tex` files decompression
#[derive(Default, Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct TexReader<T: Read> {
    reader: Reader<T>,
}

impl<T: Read> TexReader<T> {
    pub fn new(src: T) -> Self {
        Self {
            reader: Reader(src),
        }
    }

    pub fn read_header(mut self) -> Result<TexReaderWithHeader<T>, TexExtractError> {
        let mut magic_buf = [0_u8; 8];

        self.reader.read_string(&mut magic_buf)?;
        self.reader.read_pad_byte()?;
        let magic = std::str::from_utf8(&magic_buf)?;

        if magic != "TEXV0005" {
            return Err(TexExtractError::UnknownMagic);
        }

        self.reader.read_string(&mut magic_buf)?;
        self.reader.read_pad_byte()?;
        let magic = std::str::from_utf8(&magic_buf)?;

        if magic != "TEXI0001" {
            return Err(TexExtractError::UnknownMagic);
        }

        let format = TexFormat::try_from(self.reader.read_int()?)?;

        let flags = self.reader.read_int()?;
        let flags = TexFlags::from_bits(flags as u8).ok_or(TexExtractError::InvalidTexFlags)?;

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

#[derive(Default, Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct TexReaderWithHeader<T: Read> {
    reader: Reader<T>,
    header: HeaderMeta,
}

impl<T: Read> TexReaderWithHeader<T> {
    pub fn get_header(&self) -> HeaderMeta {
        self.header
    }

    pub fn read_image_container_meta(
        mut self,
    ) -> Result<TexReaderWithImageContainerMeta<T>, TexExtractError> {
        let mut magic_buf = [0_u8; 8];

        self.reader.read_string(&mut magic_buf)?;
        self.reader.read_pad_byte()?;

        let magic = std::str::from_utf8(&magic_buf)?;

        let mut version = match magic {
            "TEXB0004" => ImageContainerVersion::Texb0004,
            "TEXB0003" => ImageContainerVersion::Texb0003,
            "TEXB0002" => ImageContainerVersion::Texb0002,
            "TEXB0001" => ImageContainerVersion::Texb0001,
            _ => return Err(TexExtractError::UnknownMagic),
        };

        let image_count = self.reader.read_int()?;

        let mut image_format = None;

        if version == ImageContainerVersion::Texb0003 || version == ImageContainerVersion::Texb0004
        {
            let fif = self.reader.read_int()?;
            image_format = Some(FreeImageFormat::from(fif));
        }

        let mut is_video_mp4 = false;

        if version == ImageContainerVersion::Texb0004 {
            is_video_mp4 = self.reader.read_int()? != 0;

            if image_format.unwrap() == FreeImageFormat::Unknow && is_video_mp4 {
                image_format = Some(FreeImageFormat::Mp4)
            }

            if !is_video_mp4 {
                version = ImageContainerVersion::Texb0003;
            }
        }

        let image_container = ImageContainerMeta {
            version,
            image_count,
            image_format,
            is_video_mp4,
        };

        Ok(TexReaderWithImageContainerMeta {
            reader: self.reader,
            header: self.header,
            image_container,
        })
    }
}

#[derive(Default, Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct TexReaderWithImageContainerMeta<T: Read> {
    reader: Reader<T>,
    header: HeaderMeta,
    image_container: ImageContainerMeta,
}

impl<T: Read> TexReaderWithImageContainerMeta<T> {
    pub fn header(&self) -> HeaderMeta {
        self.header
    }
    pub fn image_container_meta(&self) -> ImageContainerMeta {
        self.image_container
    }

    pub fn read_images(mut self) -> Result<TexReaderWithImages<T>, TexExtractError> {
        let mut images = Vec::with_capacity(self.image_container.image_count as usize);

        for _ in 0..self.image_container.image_count {
            let mipmap_count = self.reader.read_int()?;

            let mut mipmaps = Vec::with_capacity(mipmap_count as usize);

            // FIXME(ArnoDarkrose): remove this take
            for _ in (0..mipmap_count).take(1) {
                let format = MipmapFormat::from_image_and_tex(
                    self.image_container.image_format,
                    self.header.format,
                );

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

                    let condition_json_cstr = self.reader.read_cstr()?;
                    condition_json = Some(condition_json_cstr.into_string()?);

                    let param3 = self.reader.read_int()?;
                    if param3 != 1 {
                        panic!("param3 is not 1, the library is likely outdated")
                    }
                }

                let width = self.reader.read_int()?;
                let height = self.reader.read_int()?;

                let lz4_compressed =
                    if self.image_container.version == ImageContainerVersion::Texb0001 {
                        false
                    } else {
                        self.reader.read_int()? == 1
                    };

                let decompressed_bytes_count =
                    if self.image_container.version == ImageContainerVersion::Texb0001 {
                        0
                    } else {
                        self.reader.read_int()?
                    };

                let byte_count = self.reader.read_int()?;

                let data = read_into_uninit(&mut self.reader.0, byte_count as usize)?;

                // FIXME(ArnoDarkrose): remove this
                // println!("uncompressed_mipmap_data len: {}, decompressed_bytes_count: {decompressed_bytes_count}", data.len());

                let data = transmute_vec(data).ok_or(TexExtractError::TransmuteError)?;

                mipmaps.push(Mipmap {
                    width,
                    height,
                    data,
                    lz4_compressed,
                    decompressed_bytes_count,
                    condition_json,
                    format,
                });
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
#[derive(Default, Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct TexReaderWithImages<T: Read> {
    reader: Reader<T>,
    header: HeaderMeta,
    image_container: ImageContainerMeta,
    images: Option<Vec<TexImage>>,
}

impl<T: Read> TexReaderWithImages<T> {
    pub fn header(&self) -> HeaderMeta {
        self.header
    }

    pub fn image_container_meta(&self) -> ImageContainerMeta {
        self.image_container
    }

    pub fn images(&mut self) -> Option<Vec<TexImage>> {
        self.images.take()
    }

    pub fn read_gif_container(mut self) -> Result<TexReaderWithGifContainer<T>, TexExtractError> {
        let mut magic_buf = [0_u8; 8];

        self.reader.read_string(&mut magic_buf)?;
        self.reader.read_pad_byte()?;

        let magic = std::str::from_utf8(&magic_buf)?;

        let version = match magic {
            "TEXS0001" => GifContainerVersion::Texs0001,
            "TEXS0002" => GifContainerVersion::Texs0002,
            "TEXS0003" => GifContainerVersion::Texs0003,
            _ => return Err(TexExtractError::UnknownMagic),
        };

        let frame_count = self.reader.read_int()?;

        let meta = GifContainerMeta {
            version,
            frame_count,
        };

        Ok(TexReaderWithGifContainer {
            reader: self.reader,
            header: self.header,
            image_container: self.image_container,
            images: self.images,
            gif_container: meta,
        })
    }
}

#[derive(Default, Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct TexReaderWithGifContainer<T: Read> {
    reader: Reader<T>,
    header: HeaderMeta,
    image_container: ImageContainerMeta,
    images: Option<Vec<TexImage>>,
    gif_container: GifContainerMeta,
}

impl<T: Read> TexReaderWithGifContainer<T> {
    pub fn header(&self) -> HeaderMeta {
        self.header
    }

    pub fn image_container(&self) -> ImageContainerMeta {
        self.image_container
    }

    pub fn images(&mut self) -> Option<Vec<TexImage>> {
        self.images.take()
    }

    pub fn gif_container(&self) -> GifContainerMeta {
        self.gif_container
    }

    pub fn read_gif_frames(mut self) -> Result<TexReaderWithGifFrames<T>, TexExtractError> {
        let gif_width = if self.gif_container.version == GifContainerVersion::Texs0003 {
            self.reader.read_int()?
        } else {
            0
        };
        let gif_height = if self.gif_container.version == GifContainerVersion::Texs0003 {
            self.reader.read_int()?
        } else {
            0
        };

        let mut frames = Vec::with_capacity(self.gif_container.frame_count as usize);

        for _ in 0..self.gif_container.frame_count {
            let image_id = self.reader.read_int()?;
            let frame_time = self.reader.read_float()?;
            let x;
            let y;
            let width;
            let width_y;
            let height_x;
            let height;

            if self.gif_container.version == GifContainerVersion::Texs0003 {
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

            frames.push(TexGifFrame {
                image_id,
                frame_time,
                x,
                y,
                width,
                width_y,
                height_x,
                height,
            });
        }

        Ok(TexReaderWithGifFrames {
            reader: self.reader,
            header: self.header,
            image_container: self.image_container,
            images: self.images,
            gif_container: self.gif_container,
            gif_frames: Some(frames),
            gif_width,
            gif_height,
        })
    }
}

#[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
pub struct TexReaderWithGifFrames<T: Read> {
    reader: Reader<T>,
    header: HeaderMeta,
    image_container: ImageContainerMeta,
    images: Option<Vec<TexImage>>,
    gif_container: GifContainerMeta,
    gif_frames: Option<Vec<TexGifFrame>>,
    // These two fiedlds make sense only for the third version of gif container
    gif_width: i32,
    gif_height: i32,
}
