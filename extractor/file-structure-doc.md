# Format info

## File structure

`.tex` file consists of the following blocks:
- `Header` which is represented by [`HeaderMeta`]
  - `8` bytes that represent ascii-encoded string that has to be `"TEXV0005"`
  - padding byte
  - `8` bytes that represent ascii-encoded string that has to be `"TEXI0001"`
  - padding byte
  - `7` [`i32`]s that represent
    - [`TexFormat`]
    - [`TexFlags`]
    - `texture_width`
    - `texture_height`
    - `image_width`
    - `image_height`
    - `unknown_int0` which is a value with non-determined purpose
- `ImageContainer`. This is a structure that contains images and all the related info
  - [`TexImageContainerMeta`]
    - `8` bytes that represent ascii-encoded string that has to be either `"TEXB0001"`,
      or `"TEXB0002"`, or `"TEXB0003"`, or `"TEXB0004"` which represent the magic(version) of the struct
    - `image_count`: [`i32`]
    - for versions `3` or `4`:
      - [`FreeImageFormat`][]: [`i32`]
    - for version `4`:
      - `is_video_mp4`: [`i32`]
  - `Images`: `image_count` images (represented by [`TexImage`]) every of which is
    - `mipmap_count`: [`i32`]
    - `Mipmaps`: mipmap count mipmaps (represented by [`TexMipmap`]) every of which is
      - for `image_container` version `1`:
        - `width`: [`i32`]
        - `height`: [`i32`]
        - `byte_count`: [`i32`]
        - `data`: `byte_count` bytes
      - for `image_container` version `2` or `3`:
        - `width`: [`i32`]
        - `height`: [`i32`]
        - `is_lz4_compressed`: [`i32`]
        - `decompresed_bytes_count`: [`i32`]
        - `byte_count`: [`i32`]
        - `data`: `byte_count` bytes
      - for `image_container `version`4`:
        - `param1`: [`i32`]. This a value of unknwon purpose
          that always equals to `1`
        - `param2`: [`i32`]. This a value of unknwon purpose
          that always equals to `2`
        - `condtion_json`: null-terminated string of ascii chars
        - `param3`: [`i32`]. This a value of unknwon purpose
          that always equals to `1`
        - rest of the parameteres are the same as for versions `2` or `3`
 - `GifContainer`. This is a structure that contains and all the gif related info
   - `GifConainerMeta` represented by [`TexGifContainerMeta`]:
     - `8` bytes that represent ascii-encoded string that has to be either `"TEXS0001"`,
       or `"TEXS0002"`, or `"TEXS0003"` which represent the magic(version) of the struct
     - `frame_count`: [`i32`]
     - for `gif_container `version `1`:
       - `FramesMeta`: `frames_count` structures with meta info for frames (represented by [`TexGifFrameMeta`])
         every of which is:
         - `image_id`: [`i32`]
         - `frame_time`: [`f32`]
         - `x`: [`i32`]
         - `y`: [`i32`]
         - `width`: [`i32`]
         - `width_y`: [`i32`]
         - `height_x`: [`i32`]
         - `height`: [`i32`]
     - for `gif_container` version `2`:
       - `FramesMeta`: frames_count structures with meta info for frames (represented by [`TexGifFrameMeta`])
         every of which is:
         - `image_id`: [`i32`]
         - `frame_time`: [`f32`]
         - `x`: [`f32`]
         - `y`: [`f32`]
         - `width`: [`f32`]
         - `width_y`: [`f32`]
         - `height_x`: [`f32`]
         - `height`: [`f32`]
      - for `gif_container` version `3`:
        - `gif_width`: `i32`
        - `gif_width`: `i32`
        - the same as for version `2`

# General info
- The format can contain images, gifs and videos
- If the contained file is video than bytes from `4` to `12` of the first mipmap of the first image
  have to form a `ftypisom`, `ftypmsnv` or `ftypmp42`
- Gif frames themeselves are stored in a single image and need to be extracted via cropping. All
  the related meta info is stored in `FramesMeta` part later
- If the format is image or video, then the `GifContainer` won't be present (so attempts to read it will lead to the `EOF` error)
