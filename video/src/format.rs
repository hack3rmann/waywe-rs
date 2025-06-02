use crate::VideoPixelDescriptor;
use bitflags::bitflags;
use ffmpeg_sys_next::{AVPixelFormat, AVSampleFormat, av_pix_fmt_desc_get};
use std::mem;

/// Audio sample formats
///
/// - The data described by the sample format is always in native-endian order.
///   Sample values can be expressed by native C types, hence the lack of a signed
///   24-bit sample format even though it is a common raw audio data format.
/// - The floating-point formats are based on full volume being in the range
///   [-1.0, 1.0](<-1.0, 1.0>). Any values outside this range are beyond full volume level.
/// - The data layout as used in av_samples_fill_arrays() and elsewhere in FFmpeg
///   (such as AVFrame in libavcodec) is as follows:
///
/// For planar sample formats, each audio channel is in a separate data plane,
/// and linesize is the buffer size, in bytes, for a single plane. All data
/// planes must be the same size. For packed sample formats, only the first data
/// plane is used, and samples for each channel are interleaved. In this case,
/// linesize is the buffer size, in bytes, for the 1 plane.
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum AudioSampleFormat {
    /// Unsigned 8 bits
    U8 = 0,
    /// Signed 16 bits
    I16 = 1,
    /// Signed 32 bits
    I32 = 2,
    /// Float
    F32 = 3,
    /// Double
    F64 = 4,
    /// Unsigned 8 bits, planar
    PlanarU8 = 5,
    /// Signed 16 bits, planar
    PlanarI16 = 6,
    /// Signed 32 bits, planar
    PlanarI32 = 7,
    /// Float, planar
    PlanarF32 = 8,
    /// Double, planar
    PlanarF64 = 9,
    /// Signed 64 bits
    I64 = 10,
    /// Signed 64 bits, planar
    PlanarI64 = 11,
}

impl AudioSampleFormat {
    pub const COUNT: usize = AVSampleFormat::AV_SAMPLE_FMT_NB as usize;

    /// Get FFI-compatible value
    pub const fn to_backend(self) -> AVSampleFormat {
        match self {
            Self::U8 => AVSampleFormat::AV_SAMPLE_FMT_U8,
            Self::I16 => AVSampleFormat::AV_SAMPLE_FMT_S16,
            Self::I32 => AVSampleFormat::AV_SAMPLE_FMT_S32,
            Self::F32 => AVSampleFormat::AV_SAMPLE_FMT_FLT,
            Self::F64 => AVSampleFormat::AV_SAMPLE_FMT_DBL,
            Self::PlanarU8 => AVSampleFormat::AV_SAMPLE_FMT_U8P,
            Self::PlanarI16 => AVSampleFormat::AV_SAMPLE_FMT_S16P,
            Self::PlanarI32 => AVSampleFormat::AV_SAMPLE_FMT_S32P,
            Self::PlanarF32 => AVSampleFormat::AV_SAMPLE_FMT_FLTP,
            Self::PlanarF64 => AVSampleFormat::AV_SAMPLE_FMT_DBLP,
            Self::I64 => AVSampleFormat::AV_SAMPLE_FMT_S64,
            Self::PlanarI64 => AVSampleFormat::AV_SAMPLE_FMT_S64P,
        }
    }

    /// Construct [`AudioSampleFormat`] from FFI-compatible value
    ///
    /// # Note
    ///
    /// Returns [`None`] if `value` is either [`AVSampleFormat::AV_SAMPLE_FMT_NONE`] or
    /// [`AVSampleFormat::AV_SAMPLE_FMT_NB`]
    pub const fn from_backend(value: AVSampleFormat) -> Option<Self> {
        Some(match value {
            AVSampleFormat::AV_SAMPLE_FMT_U8 => Self::U8,
            AVSampleFormat::AV_SAMPLE_FMT_S16 => Self::I16,
            AVSampleFormat::AV_SAMPLE_FMT_S32 => Self::I32,
            AVSampleFormat::AV_SAMPLE_FMT_FLT => Self::F32,
            AVSampleFormat::AV_SAMPLE_FMT_DBL => Self::F64,
            AVSampleFormat::AV_SAMPLE_FMT_U8P => Self::PlanarU8,
            AVSampleFormat::AV_SAMPLE_FMT_S16P => Self::PlanarI16,
            AVSampleFormat::AV_SAMPLE_FMT_S32P => Self::PlanarI32,
            AVSampleFormat::AV_SAMPLE_FMT_FLTP => Self::PlanarF32,
            AVSampleFormat::AV_SAMPLE_FMT_DBLP => Self::PlanarF64,
            AVSampleFormat::AV_SAMPLE_FMT_S64 => Self::I64,
            AVSampleFormat::AV_SAMPLE_FMT_S64P => Self::PlanarI64,
            AVSampleFormat::AV_SAMPLE_FMT_NONE | AVSampleFormat::AV_SAMPLE_FMT_NB => return None,
        })
    }

    /// Construct [`AudioSampleFormat`] from [`i32`]
    pub const fn from_i32(value: i32) -> Option<Self> {
        Some(match value {
            0 => Self::U8,
            1 => Self::I16,
            2 => Self::I32,
            3 => Self::F32,
            4 => Self::F64,
            5 => Self::PlanarU8,
            6 => Self::PlanarI16,
            7 => Self::PlanarI32,
            8 => Self::PlanarF32,
            9 => Self::PlanarF64,
            10 => Self::I64,
            11 => Self::PlanarI64,
            _ => return None,
        })
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum VideoPixelFormat {
    /// planar YUV 4:2:0, 12bpp, (1 Cr & Cb sample per 2x2 Y samples)
    Yuv420p = 0,
    /// packed YUV 4:2:2, 16bpp, Y0 Cb Y1 Cr
    Yuyv422 = 1,
    /// packed RGB 8:8:8, 24bpp, RGBRGB...
    Rgb24 = 2,
    /// packed RGB 8:8:8, 24bpp, BGRBGR...
    Bgr24 = 3,
    /// planar YUV 4:2:2, 16bpp, (1 Cr & Cb sample per 2x1 Y samples)
    Yuv422p = 4,
    /// planar YUV 4:4:4, 24bpp, (1 Cr & Cb sample per 1x1 Y samples)
    Yuv444p = 5,
    /// planar YUV 4:1:0,  9bpp, (1 Cr & Cb sample per 4x4 Y samples)
    Yuv410p = 6,
    /// planar YUV 4:1:1, 12bpp, (1 Cr & Cb sample per 4x1 Y samples)
    Yuv411p = 7,
    ///        Y        ,  8bpp
    Gray8 = 8,
    ///        Y        ,  1bpp, 0 is white, 1 is black, in each byte pixels are ordered from the msb to the lsb
    Monowhite = 9,
    ///        Y        ,  1bpp, 0 is black, 1 is white, in each byte pixels are ordered from the msb to the lsb
    Monoblack = 10,
    /// 8 bits with AV_PIX_FMT_RGB32 palette
    Pal8 = 11,
    /// planar YUV 4:2:0, 12bpp, full scale (JPEG), deprecated in favor of AV_PIX_FMT_YUV420P and setting color_range
    Yuvj420p = 12,
    /// planar YUV 4:2:2, 16bpp, full scale (JPEG), deprecated in favor of AV_PIX_FMT_YUV422P and setting color_range
    Yuvj422p = 13,
    /// planar YUV 4:4:4, 24bpp, full scale (JPEG), deprecated in favor of AV_PIX_FMT_YUV444P and setting color_range
    Yuvj444p = 14,
    /// packed YUV 4:2:2, 16bpp, Cb Y0 Cr Y1
    Uyvy422 = 15,
    /// packed YUV 4:1:1, 12bpp, Cb Y0 Y1 Cr Y2 Y3
    Uyyvyy411 = 16,
    /// packed RGB 3:3:2,  8bpp, (msb)2B 3G 3R(lsb)
    Bgr8 = 17,
    /// packed RGB 1:2:1 bitstream,  4bpp, (msb)1B 2G 1R(lsb), a byte contains two pixels, the first pixel in the byte is the one composed by the 4 msb bits
    Bgr4 = 18,
    /// packed RGB 1:2:1,  8bpp, (msb)1B 2G 1R(lsb)
    Bgr4Byte = 19,
    /// packed RGB 3:3:2,  8bpp, (msb)3R 3G 2B(lsb)
    Rgb8 = 20,
    /// packed RGB 1:2:1 bitstream,  4bpp, (msb)1R 2G 1B(lsb), a byte contains two pixels, the first pixel in the byte is the one composed by the 4 msb bits
    Rgb4 = 21,
    /// packed RGB 1:2:1,  8bpp, (msb)1R 2G 1B(lsb)
    Rgb4Byte = 22,
    /// planar YUV 4:2:0, 12bpp, 1 plane for Y and 1 plane for the UV components, which are interleaved (first byte U and the following byte V)
    Nv12 = 23,
    /// as above, but U and V bytes are swapped
    Nv21 = 24,
    /// packed ARGB 8:8:8:8, 32bpp, ARGBARGB...
    Argb = 25,
    /// packed RGBA 8:8:8:8, 32bpp, RGBARGBA...
    Rgba8 = 26,
    /// packed ABGR 8:8:8:8, 32bpp, ABGRABGR...
    Abgr = 27,
    /// packed BGRA 8:8:8:8, 32bpp, BGRABGRA...
    Bgra = 28,
    ///        Y        , 16bpp, big-endian
    Gray16be = 29,
    ///        Y        , 16bpp, little-endian
    Gray16le = 30,
    /// planar YUV 4:4:0 (1 Cr & Cb sample per 1x2 Y samples)
    Yuv440p = 31,
    /// planar YUV 4:4:0 full scale (JPEG), deprecated in favor of AV_PIX_FMT_YUV440P and setting color_range
    Yuvj440p = 32,
    /// planar YUV 4:2:0, 20bpp, (1 Cr & Cb sample per 2x2 Y & A samples)
    Yuva420p = 33,
    /// packed RGB 16:16:16, 48bpp, 16R, 16G, 16B, the 2-byte value for each R/G/B component is stored as big-endian
    Rgb48be = 34,
    /// packed RGB 16:16:16, 48bpp, 16R, 16G, 16B, the 2-byte value for each R/G/B component is stored as little-endian
    Rgb48le = 35,
    /// packed RGB 5:6:5, 16bpp, (msb)   5R 6G 5B(lsb), big-endian
    Rgb565be = 36,
    /// packed RGB 5:6:5, 16bpp, (msb)   5R 6G 5B(lsb), little-endian
    Rgb565le = 37,
    /// packed RGB 5:5:5, 16bpp, (msb)1X 5R 5G 5B(lsb), big-endian   , X=unused/undefined
    Rgb555be = 38,
    /// packed RGB 5:5:5, 16bpp, (msb)1X 5R 5G 5B(lsb), little-endian, X=unused/undefined
    Rgb555le = 39,
    /// packed BGR 5:6:5, 16bpp, (msb)   5B 6G 5R(lsb), big-endian
    Bgr565be = 40,
    /// packed BGR 5:6:5, 16bpp, (msb)   5B 6G 5R(lsb), little-endian
    Bgr565le = 41,
    /// packed BGR 5:5:5, 16bpp, (msb)1X 5B 5G 5R(lsb), big-endian   , X=unused/undefined
    Bgr555be = 42,
    /// packed BGR 5:5:5, 16bpp, (msb)1X 5B 5G 5R(lsb), little-endian, X=unused/undefined
    Bgr555le = 43,
    /// Hardware acceleration through VA-API, data[3] contains a\n  VASurfaceID.
    VaApi = 44,
    /// planar YUV 4:2:0, 24bpp, (1 Cr & Cb sample per 2x2 Y samples), little-endian
    Yuv420p16le = 45,
    /// planar YUV 4:2:0, 24bpp, (1 Cr & Cb sample per 2x2 Y samples), big-endian
    Yuv420p16be = 46,
    /// planar YUV 4:2:2, 32bpp, (1 Cr & Cb sample per 2x1 Y samples), little-endian
    Yuv422p16le = 47,
    /// planar YUV 4:2:2, 32bpp, (1 Cr & Cb sample per 2x1 Y samples), big-endian
    Yuv422p16be = 48,
    /// planar YUV 4:4:4, 48bpp, (1 Cr & Cb sample per 1x1 Y samples), little-endian
    Yuv444p16le = 49,
    /// planar YUV 4:4:4, 48bpp, (1 Cr & Cb sample per 1x1 Y samples), big-endian
    Yuv444p16be = 50,
    /// HW decoding through DXVA2, Picture.data[3] contains a LPDIRECT3DSURFACE9 pointer
    Dxva2Vld = 51,
    /// packed RGB 4:4:4, 16bpp, (msb)4X 4R 4G 4B(lsb), little-endian, X=unused/undefined
    Rgb444le = 52,
    /// packed RGB 4:4:4, 16bpp, (msb)4X 4R 4G 4B(lsb), big-endian,    X=unused/undefined
    Rgb444be = 53,
    /// packed BGR 4:4:4, 16bpp, (msb)4X 4B 4G 4R(lsb), little-endian, X=unused/undefined
    Bgr444le = 54,
    /// packed BGR 4:4:4, 16bpp, (msb)4X 4B 4G 4R(lsb), big-endian,    X=unused/undefined
    Bgr444be = 55,
    /// 8 bits gray, 8 bits alpha
    Ya8 = 56,
    /// packed RGB 16:16:16, 48bpp, 16B, 16G, 16R, the 2-byte value for each R/G/B component is stored as big-endian
    Bgr48be = 57,
    /// packed RGB 16:16:16, 48bpp, 16B, 16G, 16R, the 2-byte value for each R/G/B component is stored as little-endian
    Bgr48le = 58,
    /// planar YUV 4:2:0, 13.5bpp, (1 Cr & Cb sample per 2x2 Y samples), big-endian
    Yuv420p9be = 59,
    /// planar YUV 4:2:0, 13.5bpp, (1 Cr & Cb sample per 2x2 Y samples), little-endian
    Yuv420p9le = 60,
    /// planar YUV 4:2:0, 15bpp, (1 Cr & Cb sample per 2x2 Y samples), big-endian
    Yuv420p10be = 61,
    /// planar YUV 4:2:0, 15bpp, (1 Cr & Cb sample per 2x2 Y samples), little-endian
    Yuv420p10le = 62,
    /// planar YUV 4:2:2, 20bpp, (1 Cr & Cb sample per 2x1 Y samples), big-endian
    Yuv422p10be = 63,
    /// planar YUV 4:2:2, 20bpp, (1 Cr & Cb sample per 2x1 Y samples), little-endian
    Yuv422p10le = 64,
    /// planar YUV 4:4:4, 27bpp, (1 Cr & Cb sample per 1x1 Y samples), big-endian
    Yuv444p9be = 65,
    /// planar YUV 4:4:4, 27bpp, (1 Cr & Cb sample per 1x1 Y samples), little-endian
    Yuv444p9le = 66,
    /// planar YUV 4:4:4, 30bpp, (1 Cr & Cb sample per 1x1 Y samples), big-endian
    Yuv444p10be = 67,
    /// planar YUV 4:4:4, 30bpp, (1 Cr & Cb sample per 1x1 Y samples), little-endian
    Yuv444p10le = 68,
    /// planar YUV 4:2:2, 18bpp, (1 Cr & Cb sample per 2x1 Y samples), big-endian
    Yuv422p9be = 69,
    /// planar YUV 4:2:2, 18bpp, (1 Cr & Cb sample per 2x1 Y samples), little-endian
    Yuv422p9le = 70,
    /// planar GBR 4:4:4 24bpp
    Gbrp = 71,
    /// planar GBR 4:4:4 27bpp, big-endian
    Gbrp9be = 72,
    /// planar GBR 4:4:4 27bpp, little-endian
    Gbrp9le = 73,
    /// planar GBR 4:4:4 30bpp, big-endian
    Gbrp10be = 74,
    /// planar GBR 4:4:4 30bpp, little-endian
    Gbrp10le = 75,
    /// planar GBR 4:4:4 48bpp, big-endian
    Gbrp16be = 76,
    /// planar GBR 4:4:4 48bpp, little-endian
    Gbrp16le = 77,
    /// planar YUV 4:2:2 24bpp, (1 Cr & Cb sample per 2x1 Y & A samples)
    Yuva422p = 78,
    /// planar YUV 4:4:4 32bpp, (1 Cr & Cb sample per 1x1 Y & A samples)
    Yuva444p = 79,
    /// planar YUV 4:2:0 22.5bpp, (1 Cr & Cb sample per 2x2 Y & A samples), big-endian
    Yuva420p9be = 80,
    /// planar YUV 4:2:0 22.5bpp, (1 Cr & Cb sample per 2x2 Y & A samples), little-endian
    Yuva420p9le = 81,
    /// planar YUV 4:2:2 27bpp, (1 Cr & Cb sample per 2x1 Y & A samples), big-endian
    Yuva422p9be = 82,
    /// planar YUV 4:2:2 27bpp, (1 Cr & Cb sample per 2x1 Y & A samples), little-endian
    Yuva422p9le = 83,
    /// planar YUV 4:4:4 36bpp, (1 Cr & Cb sample per 1x1 Y & A samples), big-endian
    Yuva444p9be = 84,
    /// planar YUV 4:4:4 36bpp, (1 Cr & Cb sample per 1x1 Y & A samples), little-endian
    Yuva444p9le = 85,
    /// planar YUV 4:2:0 25bpp, (1 Cr & Cb sample per 2x2 Y & A samples, big-endian)
    Yuva420p10be = 86,
    /// planar YUV 4:2:0 25bpp, (1 Cr & Cb sample per 2x2 Y & A samples, little-endian)
    Yuva420p10le = 87,
    /// planar YUV 4:2:2 30bpp, (1 Cr & Cb sample per 2x1 Y & A samples, big-endian)
    Yuva422p10be = 88,
    /// planar YUV 4:2:2 30bpp, (1 Cr & Cb sample per 2x1 Y & A samples, little-endian)
    Yuva422p10le = 89,
    /// planar YUV 4:4:4 40bpp, (1 Cr & Cb sample per 1x1 Y & A samples, big-endian)
    Yuva444p10be = 90,
    /// planar YUV 4:4:4 40bpp, (1 Cr & Cb sample per 1x1 Y & A samples, little-endian)
    Yuva444p10le = 91,
    /// planar YUV 4:2:0 40bpp, (1 Cr & Cb sample per 2x2 Y & A samples, big-endian)
    Yuva420p16be = 92,
    /// planar YUV 4:2:0 40bpp, (1 Cr & Cb sample per 2x2 Y & A samples, little-endian)
    Yuva420p16le = 93,
    /// planar YUV 4:2:2 48bpp, (1 Cr & Cb sample per 2x1 Y & A samples, big-endian)
    Yuva422p16be = 94,
    /// planar YUV 4:2:2 48bpp, (1 Cr & Cb sample per 2x1 Y & A samples, little-endian)
    Yuva422p16le = 95,
    /// planar YUV 4:4:4 64bpp, (1 Cr & Cb sample per 1x1 Y & A samples, big-endian)
    Yuva444p16be = 96,
    /// planar YUV 4:4:4 64bpp, (1 Cr & Cb sample per 1x1 Y & A samples, little-endian)
    Yuva444p16le = 97,
    /// HW acceleration through VDPAU, Picture.data[3] contains a VdpVideoSurface
    Vdpau = 98,
    /// packed XYZ 4:4:4, 36 bpp, (msb) 12X, 12Y, 12Z (lsb), the 2-byte value for each X/Y/Z is stored as little-endian, the 4 lower bits are set to 0
    Xyz12le = 99,
    /// packed XYZ 4:4:4, 36 bpp, (msb) 12X, 12Y, 12Z (lsb), the 2-byte value for each X/Y/Z is stored as big-endian, the 4 lower bits are set to 0
    Xyz12be = 100,
    /// interleaved chroma YUV 4:2:2, 16bpp, (1 Cr & Cb sample per 2x1 Y samples)
    Nv16 = 101,
    /// interleaved chroma YUV 4:2:2, 20bpp, (1 Cr & Cb sample per 2x1 Y samples), little-endian
    Nv20le = 102,
    /// interleaved chroma YUV 4:2:2, 20bpp, (1 Cr & Cb sample per 2x1 Y samples), big-endian
    Nv20be = 103,
    /// packed RGBA 16:16:16:16, 64bpp, 16R, 16G, 16B, 16A, the 2-byte value for each R/G/B/A component is stored as big-endian
    Rgba64be = 104,
    /// packed RGBA 16:16:16:16, 64bpp, 16R, 16G, 16B, 16A, the 2-byte value for each R/G/B/A component is stored as little-endian
    Rgba64le = 105,
    /// packed RGBA 16:16:16:16, 64bpp, 16B, 16G, 16R, 16A, the 2-byte value for each R/G/B/A component is stored as big-endian
    Bgra64be = 106,
    /// packed RGBA 16:16:16:16, 64bpp, 16B, 16G, 16R, 16A, the 2-byte value for each R/G/B/A component is stored as little-endian
    Bgra64le = 107,
    /// packed YUV 4:2:2, 16bpp, Y0 Cr Y1 Cb
    Yvyu422 = 108,
    /// 16 bits gray, 16 bits alpha (big-endian)
    Ya16be = 109,
    /// 16 bits gray, 16 bits alpha (little-endian)
    Ya16le = 110,
    /// planar GBRA 4:4:4:4 32bpp
    Gbrap = 111,
    /// planar GBRA 4:4:4:4 64bpp, big-endian
    Gbrap16be = 112,
    /// planar GBRA 4:4:4:4 64bpp, little-endian
    Gbrap16le = 113,
    /// HW acceleration through QSV, data[3] contains a pointer to the\n mfxFrameSurface1 structure.\n\n Before FFmpeg 5.0:\n mfxFrameSurface1.Data.MemId contains a pointer when importing\n the following frames as QSV frames:\n\n VAAPI:\n mfxFrameSurface1.Data.MemId contains a pointer to VASurfaceID\n\n DXVA2:\n mfxFrameSurface1.Data.MemId contains a pointer to IDirect3DSurface9\n\n FFmpeg 5.0 and above:\n mfxFrameSurface1.Data.MemId contains a pointer to the mfxHDLPair\n structure when importing the following frames as QSV frames:\n\n VAAPI:\n mfxHDLPair.first contains a VASurfaceID pointer.\n mfxHDLPair.second is always MFX_INFINITE.\n\n DXVA2:\n mfxHDLPair.first contains IDirect3DSurface9 pointer.\n mfxHDLPair.second is always MFX_INFINITE.\n\n D3D11:\n mfxHDLPair.first contains a ID3D11Texture2D pointer.\n mfxHDLPair.second contains the texture array index of the frame if the\n ID3D11Texture2D is an array texture, or always MFX_INFINITE if it is a\n normal texture.
    Qsv = 114,
    /// HW acceleration though MMAL, data[3] contains a pointer to the\n MMAL_BUFFER_HEADER_T structure.
    Mmal = 115,
    /// HW decoding through Direct3D11 via old API, Picture.data[3] contains a ID3D11VideoDecoderOutputView pointer
    D3d11vaVld = 116,
    /// HW acceleration through CUDA. data[i] contain CUdeviceptr pointers\n exactly as for system memory frames.
    Cuda = 117,
    /// packed RGB 8:8:8, 32bpp, XRGBXRGB...   X=unused/undefined
    _0rgb = 118,
    /// packed RGB 8:8:8, 32bpp, RGBXRGBX...   X=unused/undefined
    Rgb0 = 119,
    /// packed BGR 8:8:8, 32bpp, XBGRXBGR...   X=unused/undefined
    _0bgr = 120,
    /// packed BGR 8:8:8, 32bpp, BGRXBGRX...   X=unused/undefined
    Bgr0 = 121,
    /// planar YUV 4:2:0,18bpp, (1 Cr & Cb sample per 2x2 Y samples), big-endian
    Yuv420p12be = 122,
    /// planar YUV 4:2:0,18bpp, (1 Cr & Cb sample per 2x2 Y samples), little-endian
    Yuv420p12le = 123,
    /// planar YUV 4:2:0,21bpp, (1 Cr & Cb sample per 2x2 Y samples), big-endian
    Yuv420p14be = 124,
    /// planar YUV 4:2:0,21bpp, (1 Cr & Cb sample per 2x2 Y samples), little-endian
    Yuv420p14le = 125,
    /// planar YUV 4:2:2,24bpp, (1 Cr & Cb sample per 2x1 Y samples), big-endian
    Yuv422p12be = 126,
    /// planar YUV 4:2:2,24bpp, (1 Cr & Cb sample per 2x1 Y samples), little-endian
    Yuv422p12le = 127,
    /// planar YUV 4:2:2,28bpp, (1 Cr & Cb sample per 2x1 Y samples), big-endian
    Yuv422p14be = 128,
    /// planar YUV 4:2:2,28bpp, (1 Cr & Cb sample per 2x1 Y samples), little-endian
    Yuv422p14le = 129,
    /// planar YUV 4:4:4,36bpp, (1 Cr & Cb sample per 1x1 Y samples), big-endian
    Yuv444p12be = 130,
    /// planar YUV 4:4:4,36bpp, (1 Cr & Cb sample per 1x1 Y samples), little-endian
    Yuv444p12le = 131,
    /// planar YUV 4:4:4,42bpp, (1 Cr & Cb sample per 1x1 Y samples), big-endian
    Yuv444p14be = 132,
    /// planar YUV 4:4:4,42bpp, (1 Cr & Cb sample per 1x1 Y samples), little-endian
    Yuv444p14le = 133,
    /// planar GBR 4:4:4 36bpp, big-endian
    Gbrp12be = 134,
    /// planar GBR 4:4:4 36bpp, little-endian
    Gbrp12le = 135,
    /// planar GBR 4:4:4 42bpp, big-endian
    Gbrp14be = 136,
    /// planar GBR 4:4:4 42bpp, little-endian
    Gbrp14le = 137,
    /// planar YUV 4:1:1, 12bpp, (1 Cr & Cb sample per 4x1 Y samples) full scale (JPEG), deprecated in favor of AV_PIX_FMT_YUV411P and setting color_range
    Yuvj411p = 138,
    /// bayer, BGBG..(odd line), GRGR..(even line), 8-bit samples
    BayerBggr8 = 139,
    /// bayer, RGRG..(odd line), GBGB..(even line), 8-bit samples
    BayerRggb8 = 140,
    /// bayer, GBGB..(odd line), RGRG..(even line), 8-bit samples
    BayerGbrg8 = 141,
    /// bayer, GRGR..(odd line), BGBG..(even line), 8-bit samples
    BayerGrbg8 = 142,
    /// bayer, BGBG..(odd line), GRGR..(even line), 16-bit samples, little-endian
    BayerBggr16le = 143,
    /// bayer, BGBG..(odd line), GRGR..(even line), 16-bit samples, big-endian
    BayerBggr16be = 144,
    /// bayer, RGRG..(odd line), GBGB..(even line), 16-bit samples, little-endian
    BayerRggb16le = 145,
    /// bayer, RGRG..(odd line), GBGB..(even line), 16-bit samples, big-endian
    BayerRggb16be = 146,
    /// bayer, GBGB..(odd line), RGRG..(even line), 16-bit samples, little-endian
    BayerGbrg16le = 147,
    /// bayer, GBGB..(odd line), RGRG..(even line), 16-bit samples, big-endian
    BayerGbrg16be = 148,
    /// bayer, GRGR..(odd line), BGBG..(even line), 16-bit samples, little-endian
    BayerGrbg16le = 149,
    /// bayer, GRGR..(odd line), BGBG..(even line), 16-bit samples, big-endian
    BayerGrbg16be = 150,
    /// planar YUV 4:4:0,20bpp, (1 Cr & Cb sample per 1x2 Y samples), little-endian
    Yuv440p10le = 151,
    /// planar YUV 4:4:0,20bpp, (1 Cr & Cb sample per 1x2 Y samples), big-endian
    Yuv440p10be = 152,
    /// planar YUV 4:4:0,24bpp, (1 Cr & Cb sample per 1x2 Y samples), little-endian
    Yuv440p12le = 153,
    /// planar YUV 4:4:0,24bpp, (1 Cr & Cb sample per 1x2 Y samples), big-endian
    Yuv440p12be = 154,
    /// packed AYUV 4:4:4,64bpp (1 Cr & Cb sample per 1x1 Y & A samples), little-endian
    Ayuv64le = 155,
    /// packed AYUV 4:4:4,64bpp (1 Cr & Cb sample per 1x1 Y & A samples), big-endian
    Ayuv64be = 156,
    /// hardware decoding through Videotoolbox
    Videotoolbox = 157,
    /// like NV12, with 10bpp per component, data in the high bits, zeros in the low bits, little-endian
    P010le = 158,
    /// like NV12, with 10bpp per component, data in the high bits, zeros in the low bits, big-endian
    P010be = 159,
    /// planar GBR 4:4:4:4 48bpp, big-endian
    Gbrap12be = 160,
    /// planar GBR 4:4:4:4 48bpp, little-endian
    Gbrap12le = 161,
    /// planar GBR 4:4:4:4 40bpp, big-endian
    Gbrap10be = 162,
    /// planar GBR 4:4:4:4 40bpp, little-endian
    Gbrap10le = 163,
    /// hardware decoding through MediaCodec
    Mediacodec = 164,
    ///        Y        , 12bpp, big-endian
    Gray12be = 165,
    ///        Y        , 12bpp, little-endian
    Gray12le = 166,
    ///        Y        , 10bpp, big-endian
    Gray10be = 167,
    ///        Y        , 10bpp, little-endian
    Gray10le = 168,
    /// like NV12, with 16bpp per component, little-endian
    P016le = 169,
    /// like NV12, with 16bpp per component, big-endian
    P016be = 170,
    /// Hardware surfaces for Direct3D11.\n\n This is preferred over the legacy AV_PIX_FMT_D3D11VA_VLD. The new D3D11\n hwaccel API and filtering support AV_PIX_FMT_D3D11 only.\n\n data[0] contains a ID3D11Texture2D pointer, and data[1] contains the\n texture array index of the frame as intptr_t if the ID3D11Texture2D is\n an array texture (or always 0 if it's a normal texture).
    D3D11 = 171,
    ///        Y        , 9bpp, big-endian
    Gray9be = 172,
    ///        Y        , 9bpp, little-endian
    Gray9le = 173,
    /// IEEE-754 single precision planar GBR 4:4:4,     96bpp, big-endian
    Gbrpf32be = 174,
    /// IEEE-754 single precision planar GBR 4:4:4,     96bpp, little-endian
    Gbrpf32le = 175,
    /// IEEE-754 single precision planar GBRA 4:4:4:4, 128bpp, big-endian
    Gbrapf32be = 176,
    /// IEEE-754 single precision planar GBRA 4:4:4:4, 128bpp, little-endian
    Gbrapf32le = 177,
    /// DRM-managed buffers exposed through PRIME buffer sharing.\n\n data[0] points to an AVDRMFrameDescriptor.
    DrmPrime = 178,
    /// Hardware surfaces for OpenCL.\n\n data[i] contain 2D image objects (typed in C as cl_mem, used\n in OpenCL as image2d_t) for each plane of the surface.
    OpenCl = 179,
    ///        Y        , 14bpp, big-endian
    Gray14be = 180,
    ///        Y        , 14bpp, little-endian
    Gray14le = 181,
    /// IEEE-754 single precision Y, 32bpp, big-endian
    Grayf32be = 182,
    /// IEEE-754 single precision Y, 32bpp, little-endian
    Grayf32le = 183,
    /// planar YUV 4:2:2,24bpp, (1 Cr & Cb sample per 2x1 Y samples), 12b alpha, big-endian
    Yuva422p12be = 184,
    /// planar YUV 4:2:2,24bpp, (1 Cr & Cb sample per 2x1 Y samples), 12b alpha, little-endian
    Yuva422p12le = 185,
    /// planar YUV 4:4:4,36bpp, (1 Cr & Cb sample per 1x1 Y samples), 12b alpha, big-endian
    Yuva444p12be = 186,
    /// planar YUV 4:4:4,36bpp, (1 Cr & Cb sample per 1x1 Y samples), 12b alpha, little-endian
    Yuva444p12le = 187,
    /// planar YUV 4:4:4, 24bpp, 1 plane for Y and 1 plane for the UV components, which are interleaved (first byte U and the following byte V)
    Nv24 = 188,
    /// as above, but U and V bytes are swapped
    Nv42 = 189,
    /// Vulkan hardware images.\n\n data[0] points to an AVVkFrame
    Vulkan = 190,
    /// packed YUV 4:2:2 like YUYV422, 20bpp, data in the high bits, big-endian
    Y210be = 191,
    /// packed YUV 4:2:2 like YUYV422, 20bpp, data in the high bits, little-endian
    Y210le = 192,
    /// packed RGB 10:10:10, 30bpp, (msb)2X 10R 10G 10B(lsb), little-endian, X=unused/undefined
    X2rgb10le = 193,
    /// packed RGB 10:10:10, 30bpp, (msb)2X 10R 10G 10B(lsb), big-endian, X=unused/undefined
    X2rgb10be = 194,
    /// packed BGR 10:10:10, 30bpp, (msb)2X 10B 10G 10R(lsb), little-endian, X=unused/undefined
    X2bgr10le = 195,
    /// packed BGR 10:10:10, 30bpp, (msb)2X 10B 10G 10R(lsb), big-endian, X=unused/undefined
    X2bgr10be = 196,
    /// interleaved chroma YUV 4:2:2, 20bpp, data in the high bits, big-endian
    P210be = 197,
    /// interleaved chroma YUV 4:2:2, 20bpp, data in the high bits, little-endian
    P210le = 198,
    /// interleaved chroma YUV 4:4:4, 30bpp, data in the high bits, big-endian
    P410be = 199,
    /// interleaved chroma YUV 4:4:4, 30bpp, data in the high bits, little-endian
    P410le = 200,
    /// interleaved chroma YUV 4:2:2, 32bpp, big-endian
    P216be = 201,
    /// interleaved chroma YUV 4:2:2, 32bpp, little-endian
    P216le = 202,
    /// interleaved chroma YUV 4:4:4, 48bpp, big-endian
    P416be = 203,
    /// interleaved chroma YUV 4:4:4, 48bpp, little-endian
    P416le = 204,
    /// packed VUYA 4:4:4, 32bpp, VUYAVUYA...
    Vuya = 205,
    /// IEEE-754 half precision packed RGBA 16:16:16:16, 64bpp, RGBARGBA..., big-endian
    Rgbaf16be = 206,
    /// IEEE-754 half precision packed RGBA 16:16:16:16, 64bpp, RGBARGBA..., little-endian
    Rgbaf16le = 207,
    /// packed VUYX 4:4:4, 32bpp, Variant of VUYA where alpha channel is left undefined
    Vuyx = 208,
    /// like NV12, with 12bpp per component, data in the high bits, zeros in the low bits, little-endian
    P012le = 209,
    /// like NV12, with 12bpp per component, data in the high bits, zeros in the low bits, big-endian
    P012be = 210,
    /// packed YUV 4:2:2 like YUYV422, 24bpp, data in the high bits, zeros in the low bits, big-endian
    Y212be = 211,
    /// packed YUV 4:2:2 like YUYV422, 24bpp, data in the high bits, zeros in the low bits, little-endian
    Y212le = 212,
    /// packed XVYU 4:4:4, 32bpp, (msb)2X 10V 10Y 10U(lsb), big-endian, variant of Y410 where alpha channel is left undefined
    Xv30be = 213,
    /// packed XVYU 4:4:4, 32bpp, (msb)2X 10V 10Y 10U(lsb), little-endian, variant of Y410 where alpha channel is left undefined
    Xv30le = 214,
    /// packed XVYU 4:4:4, 48bpp, data in the high bits, zeros in the low bits, big-endian, variant of Y412 where alpha channel is left undefined
    Xv36be = 215,
    /// packed XVYU 4:4:4, 48bpp, data in the high bits, zeros in the low bits, little-endian, variant of Y412 where alpha channel is left undefined
    Xv36le = 216,
    /// IEEE-754 single precision packed RGB 32:32:32, 96bpp, RGBRGB..., big-endian
    Rgbf32be = 217,
    /// IEEE-754 single precision packed RGB 32:32:32, 96bpp, RGBRGB..., little-endian
    Rgbf32le = 218,
    /// IEEE-754 single precision packed RGBA 32:32:32:32, 128bpp, RGBARGBA..., big-endian
    Rgbaf32be = 219,
    /// IEEE-754 single precision packed RGBA 32:32:32:32, 128bpp, RGBARGBA..., little-endian
    Rgbaf32le = 220,
    /// interleaved chroma YUV 4:2:2, 24bpp, data in the high bits, big-endian
    P212be = 221,
    /// interleaved chroma YUV 4:2:2, 24bpp, data in the high bits, little-endian
    P212le = 222,
    /// interleaved chroma YUV 4:4:4, 36bpp, data in the high bits, big-endian
    P412be = 223,
    /// interleaved chroma YUV 4:4:4, 36bpp, data in the high bits, little-endian
    P412le = 224,
    /// planar GBR 4:4:4:4 56bpp, big-endian
    Gbrap14be = 225,
    /// planar GBR 4:4:4:4 56bpp, little-endian
    Gbrap14le = 226,
    /// Hardware surfaces for Direct3D 12.\n\n data[0] points to an AVD3D12VAFrame
    D3D12 = 227,
}

impl VideoPixelFormat {
    pub const COUNT: usize = AVPixelFormat::AV_PIX_FMT_NB as usize;

    /// Get FFI-compatible value
    pub const fn to_backend(self) -> AVPixelFormat {
        unsafe { mem::transmute::<Self, AVPixelFormat>(self) }
    }

    /// Construct [`VideoPixelFormat`] from FFI-compatible value
    ///
    /// # Note
    ///
    /// Returns [`None`] if `value` is either [`AVPixelFormat::AV_PIX_FMT_NONE`] or
    /// [`AVPixelFormat::AV_PIX_FMT_NB`]
    pub const fn from_backend(value: AVPixelFormat) -> Option<Self> {
        Some(match value {
            AVPixelFormat::AV_PIX_FMT_NONE | AVPixelFormat::AV_PIX_FMT_NB => return None,
            some => unsafe { mem::transmute::<AVPixelFormat, Self>(some) },
        })
    }

    pub const fn from_i32(value: i32) -> Option<Self> {
        const COUNT: i32 = VideoPixelFormat::COUNT as i32;

        match value {
            in_bounds @ 0..COUNT => Some(unsafe { mem::transmute::<i32, Self>(in_bounds) }),
            ..0 | COUNT.. => None,
        }
    }

    pub fn descriptor(self) -> Option<&'static VideoPixelDescriptor> {
        let ptr = unsafe { av_pix_fmt_desc_get(self.to_backend()) };
        unsafe { ptr.cast::<VideoPixelDescriptor>().as_ref() }
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Default, Hash)]
    pub struct PixelFormatFlags: u64 {
        const BE = 1;
        const PAL = 2;
        const BITSTREAM = 4;
        const HWACCEL = 8;
        const PLANAR = 16;
        const RGB = 32;
        const ALPHA = 128;
        const BAYER = 256;
        const FLOAT = 512;
        const XYZ = 1024;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AudioVideoFormat {
    Audio(AudioSampleFormat),
    Video(VideoPixelFormat),
}

impl AudioVideoFormat {
    pub const fn audio_from_i32(value: i32) -> Option<Self> {
        Some(match AudioSampleFormat::from_i32(value) {
            None => return None,
            Some(format) => Self::Audio(format),
        })
    }

    pub const fn video_from_i32(value: i32) -> Option<Self> {
        Some(match VideoPixelFormat::from_i32(value) {
            None => return None,
            Some(format) => Self::Video(format),
        })
    }
}
