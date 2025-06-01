use bitflags::bitflags;
use ffmpeg_next::ffi::{
    AV_PROFILE_UNKNOWN, AVCodec, AVCodecContext, AVCodecID, AVCodecParameters,
    AVERROR_BSF_NOT_FOUND, AVERROR_BUFFER_TOO_SMALL, AVERROR_BUG, AVERROR_BUG2,
    AVERROR_DECODER_NOT_FOUND, AVERROR_DEMUXER_NOT_FOUND, AVERROR_ENCODER_NOT_FOUND, AVERROR_EOF,
    AVERROR_EXIT, AVERROR_EXTERNAL, AVERROR_FILTER_NOT_FOUND, AVERROR_HTTP_BAD_REQUEST,
    AVERROR_HTTP_FORBIDDEN, AVERROR_HTTP_NOT_FOUND, AVERROR_HTTP_OTHER_4XX,
    AVERROR_HTTP_SERVER_ERROR, AVERROR_HTTP_TOO_MANY_REQUESTS, AVERROR_HTTP_UNAUTHORIZED,
    AVERROR_INVALIDDATA, AVERROR_MUXER_NOT_FOUND, AVERROR_OPTION_NOT_FOUND, AVERROR_PATCHWELCOME,
    AVERROR_PROTOCOL_NOT_FOUND, AVERROR_STREAM_NOT_FOUND, AVERROR_UNKNOWN, AVFormatContext,
    AVFrame, AVHWDeviceType, AVMediaType, AVPacket, AVPixFmtDescriptor, AVPixelFormat, AVProfile,
    AVRational, AVSampleFormat, AVStream, SWS_ACCURATE_RND, SWS_AREA, SWS_BICUBIC, SWS_BICUBLIN,
    SWS_BILINEAR, SWS_BITEXACT, SWS_DIRECT_BGR, SWS_ERROR_DIFFUSION, SWS_FAST_BILINEAR,
    SWS_FULL_CHR_H_INP, SWS_FULL_CHR_H_INT, SWS_GAUSS, SWS_LANCZOS, SWS_PARAM_DEFAULT, SWS_POINT,
    SWS_PRINT_INFO, SWS_SINC, SWS_SPLINE, SWS_SRC_V_CHR_DROP_MASK, SWS_SRC_V_CHR_DROP_SHIFT, SWS_X,
    SwsContext, av_codec_is_decoder, av_codec_is_encoder, av_codec_iterate, av_find_best_stream,
    av_frame_alloc, av_frame_free, av_frame_get_buffer, av_hwdevice_get_type_name,
    av_hwdevice_iterate_types, av_new_packet, av_packet_alloc, av_packet_free, av_packet_ref,
    av_packet_unref, av_pix_fmt_desc_get, av_read_frame, avcodec_alloc_context3,
    avcodec_find_decoder, avcodec_open2, avcodec_parameters_to_context, avcodec_receive_frame,
    avcodec_send_packet, avformat_close_input, avformat_find_stream_info, avformat_open_input,
    sws_getContext, sws_scale,
};
use glam::UVec2;
use std::{
    error::Error,
    ffi::{CStr, c_void},
    fmt, hint,
    marker::PhantomData,
    mem,
    num::{NonZeroI32, NonZeroI64, NonZeroU32, NonZeroU64},
    ptr::{self, NonNull},
    slice, str,
};

#[derive(Clone, Copy, Debug)]
pub enum HwDeviceType {
    VdPau = 1,
    Cuda = 2,
    VaApi = 3,
    DxVa2 = 4,
    Qsv = 5,
    VideoToolbox = 6,
    D3D11Va = 7,
    Drm = 8,
    OpenCl = 9,
    MediaCodec = 10,
    Vulkan = 11,
    D3D12Va = 12,
}

impl HwDeviceType {
    pub const fn from_backend(ty: AVHWDeviceType) -> Option<Self> {
        Some(match ty {
            AVHWDeviceType::AV_HWDEVICE_TYPE_NONE => return None,
            AVHWDeviceType::AV_HWDEVICE_TYPE_VDPAU => Self::VdPau,
            AVHWDeviceType::AV_HWDEVICE_TYPE_CUDA => Self::Cuda,
            AVHWDeviceType::AV_HWDEVICE_TYPE_VAAPI => Self::VaApi,
            AVHWDeviceType::AV_HWDEVICE_TYPE_DXVA2 => Self::DxVa2,
            AVHWDeviceType::AV_HWDEVICE_TYPE_QSV => Self::Qsv,
            AVHWDeviceType::AV_HWDEVICE_TYPE_VIDEOTOOLBOX => Self::VideoToolbox,
            AVHWDeviceType::AV_HWDEVICE_TYPE_D3D11VA => Self::D3D11Va,
            AVHWDeviceType::AV_HWDEVICE_TYPE_DRM => Self::Drm,
            AVHWDeviceType::AV_HWDEVICE_TYPE_OPENCL => Self::OpenCl,
            AVHWDeviceType::AV_HWDEVICE_TYPE_MEDIACODEC => Self::MediaCodec,
            AVHWDeviceType::AV_HWDEVICE_TYPE_VULKAN => Self::Vulkan,
            AVHWDeviceType::AV_HWDEVICE_TYPE_D3D12VA => Self::D3D12Va,
        })
    }

    pub const fn to_backend(self) -> AVHWDeviceType {
        match self {
            Self::VdPau => AVHWDeviceType::AV_HWDEVICE_TYPE_VDPAU,
            Self::Cuda => AVHWDeviceType::AV_HWDEVICE_TYPE_CUDA,
            Self::VaApi => AVHWDeviceType::AV_HWDEVICE_TYPE_VAAPI,
            Self::DxVa2 => AVHWDeviceType::AV_HWDEVICE_TYPE_DXVA2,
            Self::Qsv => AVHWDeviceType::AV_HWDEVICE_TYPE_QSV,
            Self::VideoToolbox => AVHWDeviceType::AV_HWDEVICE_TYPE_VIDEOTOOLBOX,
            Self::D3D11Va => AVHWDeviceType::AV_HWDEVICE_TYPE_D3D11VA,
            Self::Drm => AVHWDeviceType::AV_HWDEVICE_TYPE_DRM,
            Self::OpenCl => AVHWDeviceType::AV_HWDEVICE_TYPE_OPENCL,
            Self::MediaCodec => AVHWDeviceType::AV_HWDEVICE_TYPE_MEDIACODEC,
            Self::Vulkan => AVHWDeviceType::AV_HWDEVICE_TYPE_VULKAN,
            Self::D3D12Va => AVHWDeviceType::AV_HWDEVICE_TYPE_D3D12VA,
        }
    }

    pub fn name(self) -> &'static CStr {
        let name_ptr = unsafe { av_hwdevice_get_type_name(self.into()) };
        unsafe { CStr::from_ptr(name_ptr) }
    }
}

impl From<HwDeviceType> for AVHWDeviceType {
    fn from(value: HwDeviceType) -> Self {
        value.to_backend()
    }
}

pub struct HwDeviceTypeIterator {
    ty: AVHWDeviceType,
}

impl HwDeviceTypeIterator {
    pub const fn new() -> Self {
        Self {
            ty: AVHWDeviceType::AV_HWDEVICE_TYPE_NONE,
        }
    }
}

impl Default for HwDeviceTypeIterator {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for HwDeviceTypeIterator {
    type Item = HwDeviceType;

    fn next(&mut self) -> Option<Self::Item> {
        self.ty = unsafe { av_hwdevice_iterate_types(self.ty) };
        HwDeviceType::from_backend(self.ty)
    }
}

macro_rules! implement_raw {
    ( $Wrapper:ty { $raw:ident } : $Raw:ty ) => {
        impl $Wrapper {
            // TODO(hack3rmann): safety
            /// # Safety
            pub const unsafe fn from_raw(raw: ::std::ptr::NonNull<$Raw>) -> Self {
                Self { $raw: raw }
            }

            #[allow(forgetting_copy_types, clippy::forget_non_drop)]
            pub const fn into_raw(self) -> ::std::ptr::NonNull<$Raw> {
                let result = self.$raw;
                ::std::mem::forget(self);
                result
            }

            pub const fn as_raw(&self) -> ::std::ptr::NonNull<$Raw> {
                self.$raw
            }
        }
    };
    ( $Wrapper:ty : $Raw:ty ) => {
        implement_raw!( $Wrapper { raw } : $Raw);
    };
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default, Eq, Ord, Hash)]
pub enum MediaType {
    #[default]
    Video = 0,
    Audio = 1,
    Data = 2,
    Subtitle = 3,
    Attachment = 4,
}

impl MediaType {
    pub const COUNT: usize = AVMediaType::AVMEDIA_TYPE_NB as usize;

    pub const fn from_backend(value: AVMediaType) -> Option<Self> {
        Some(match value {
            AVMediaType::AVMEDIA_TYPE_UNKNOWN => return None,
            AVMediaType::AVMEDIA_TYPE_VIDEO => Self::Video,
            AVMediaType::AVMEDIA_TYPE_AUDIO => Self::Audio,
            AVMediaType::AVMEDIA_TYPE_DATA => Self::Data,
            AVMediaType::AVMEDIA_TYPE_SUBTITLE => Self::Subtitle,
            AVMediaType::AVMEDIA_TYPE_ATTACHMENT => Self::Attachment,
            AVMediaType::AVMEDIA_TYPE_NB => return None,
        })
    }

    pub const fn to_backend(self) -> AVMediaType {
        match self {
            MediaType::Video => AVMediaType::AVMEDIA_TYPE_VIDEO,
            MediaType::Audio => AVMediaType::AVMEDIA_TYPE_AUDIO,
            MediaType::Data => AVMediaType::AVMEDIA_TYPE_DATA,
            MediaType::Subtitle => AVMediaType::AVMEDIA_TYPE_SUBTITLE,
            MediaType::Attachment => AVMediaType::AVMEDIA_TYPE_ATTACHMENT,
        }
    }
}

pub struct FormatContext {
    raw: NonNull<AVFormatContext>,
}

implement_raw!(FormatContext: AVFormatContext);

impl FormatContext {
    pub fn from_input(path: &CStr) -> Result<Self, BackendError> {
        let mut context_ptr = ptr::null_mut();

        BackendError::result_of(unsafe {
            avformat_open_input(
                &raw mut context_ptr,
                path.as_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        })?;

        let mut context = unsafe { Self::from_raw(NonNull::new_unchecked(context_ptr)) };
        context.find_stream_info()?;
        Ok(context)
    }

    pub fn find_stream_info(&mut self) -> Result<(), BackendError> {
        BackendError::result_of(unsafe {
            avformat_find_stream_info(self.as_raw().as_ptr(), ptr::null_mut())
        })
    }

    pub const fn streams(&self) -> &[Stream] {
        let ptr = unsafe { (*self.as_raw().as_ptr()).streams };
        let len = unsafe { (*self.as_raw().as_ptr()).nb_streams };
        // TODO(hack3rmann): safety
        unsafe { slice::from_raw_parts(ptr.cast(), len as usize) }
    }

    pub fn find_best_stream(&self, media_type: MediaType) -> Result<&Stream, BackendError> {
        let index = match BackendError::result_or_non_negative(unsafe {
            av_find_best_stream(
                self.as_raw().as_ptr(),
                media_type.to_backend(),
                -1,
                -1,
                ptr::null_mut(),
                0,
            )
        }) {
            Err(
                error @ BackendError::STREAM_NOT_FOUND | error @ BackendError::DECODER_NOT_FOUND,
            ) => return Err(error),
            Err(..) => unsafe { hint::unreachable_unchecked() },
            Ok(index) => index as usize,
        };

        Ok(unsafe { self.streams().get_unchecked(index) })
    }

    pub const fn bit_rate(&self) -> Option<NonZeroI64> {
        NonZeroI64::new(unsafe { (*self.as_raw().as_ptr()).bit_rate })
    }

    pub fn read_packet(&mut self, packet: &mut Packet) -> Result<(), BackendError> {
        BackendError::result_of(unsafe {
            av_read_frame(self.as_raw().as_ptr(), packet.as_raw().as_ptr())
        })
    }
}

impl Drop for FormatContext {
    fn drop(&mut self) {
        let mut ptr = self.raw.as_ptr();
        unsafe { avformat_close_input(&raw mut ptr) };
    }
}

#[repr(C)]
pub struct Stream {
    raw: NonNull<AVStream>,
}

implement_raw!(Stream: AVStream);

impl Stream {
    pub const fn index(&self) -> usize {
        match unsafe { (*self.as_raw().as_ptr()).index } {
            ..0 => unsafe { hint::unreachable_unchecked() },
            non_negative @ 0.. => non_negative as usize,
        }
    }

    pub const fn frame_count(&self) -> Option<NonZeroU64> {
        match unsafe { (*self.as_raw().as_ptr()).nb_frames } {
            ..0 => unsafe { hint::unreachable_unchecked() },
            non_negative @ 0.. => NonZeroU64::new(non_negative as u64),
        }
    }

    // TODO(hack3rmann): figure out safety for this
    pub const fn codec_parameters(&self) -> &CodecParameters {
        unsafe {
            (*self.as_raw().as_ptr())
                .codecpar
                .cast::<CodecParameters>()
                .as_ref()
                .unwrap_unchecked()
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct CodecId(pub AVCodecID);

impl fmt::Debug for CodecId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

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

    pub const fn to_backend(self) -> AVPixelFormat {
        unsafe { mem::transmute::<Self, AVPixelFormat>(self) }
    }

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

    pub fn descriptor(self) -> Option<VideoPixelDescriptor> {
        let ptr = unsafe { av_pix_fmt_desc_get(self.to_backend()) };

        NonNull::new(ptr.cast_mut()).map(|raw| unsafe { VideoPixelDescriptor::from_raw(raw) })
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Default, Hash)]
    pub struct PixelFormatFlags: u64 {
        const AV_PIX_FMT_FLAG_BE = 1;
        const AV_PIX_FMT_FLAG_PAL = 2;
        const AV_PIX_FMT_FLAG_BITSTREAM = 4;
        const AV_PIX_FMT_FLAG_HWACCEL = 8;
        const AV_PIX_FMT_FLAG_PLANAR = 16;
        const AV_PIX_FMT_FLAG_RGB = 32;
        const AV_PIX_FMT_FLAG_ALPHA = 128;
        const AV_PIX_FMT_FLAG_BAYER = 256;
        const AV_PIX_FMT_FLAG_FLOAT = 512;
        const AV_PIX_FMT_FLAG_XYZ = 1024;
    }
}

pub struct VideoPixelDescriptor {
    raw: NonNull<AVPixFmtDescriptor>,
}

implement_raw!(VideoPixelDescriptor: AVPixFmtDescriptor);

impl VideoPixelDescriptor {
    pub const fn name(&self) -> &'static str {
        let name_cstr = unsafe { CStr::from_ptr((*self.as_raw().as_ptr()).name) };
        unsafe { str::from_utf8_unchecked(name_cstr.to_bytes()) }
    }

    /// Amount to shift the luma width right to find the chroma width.
    /// For YV12 this is 1 for example.
    /// This value only refers to the chroma components.
    pub const fn log2_chroma_w(&self) -> u8 {
        unsafe { (*self.as_raw().as_ptr()).log2_chroma_w }
    }

    /// Amount to shift the luma height right to find the chroma height.
    /// For YV12 this is 1 for example.
    /// This value only refers to the chroma components.
    pub const fn log2_chroma_h(&self) -> u8 {
        unsafe { (*self.as_raw().as_ptr()).log2_chroma_h }
    }

    pub const fn flags(&self) -> PixelFormatFlags {
        let bits = unsafe { (*self.as_raw().as_ptr()).flags };
        unsafe { PixelFormatFlags::from_bits(bits).unwrap_unchecked() }
    }

    /// Alternative comma-separated names.
    pub const fn alias(&self) -> Option<&'static str> {
        let ptr = unsafe { (*self.as_raw().as_ptr()).alias };

        if ptr.is_null() {
            return None;
        }

        let alias_cstr = unsafe { CStr::from_ptr(ptr) };

        Some(unsafe { str::from_utf8_unchecked(alias_cstr.to_bytes()) })
    }

    /// Parameters that describe how pixels are packed.
    ///
    /// If the format has 1 or 2 components, then luma is 0.
    /// If the format has 3 or 4 components:
    /// if the RGB flag is set then 0 is red, 1 is green and 2 is blue;
    /// otherwise 0 is luma, 1 is chroma-U and 2 is chroma-V.
    ///
    /// If present, the Alpha channel is always the last component.
    pub const fn components(&self) -> &[ComponentDescriptor] {
        let ptr = unsafe { &raw const (*self.as_raw().as_ptr()).comp[0] };
        let count = unsafe { (*self.as_raw().as_ptr()).nb_components };
        unsafe { slice::from_raw_parts(ptr, count as usize) }
    }
}

impl fmt::Debug for VideoPixelDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VideoPixelDescriptor")
            .field("name", &self.name())
            .field("log2_chroma_w", &self.log2_chroma_w())
            .field("log2_chroma_h", &self.log2_chroma_h())
            .field("flags", &self.flags())
            .field("alias", &self.alias())
            .field("components", &self.components())
            .finish()
    }
}

pub use ffmpeg_next::ffi::AVComponentDescriptor as ComponentDescriptor;

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

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct RatioI32 {
    pub numerator: i32,
    pub denominator: NonZeroI32,
}

impl RatioI32 {
    pub const ZERO: Self = Self {
        numerator: 0,
        denominator: NonZeroI32::new(1).unwrap(),
    };

    pub const ONE: Self = Self {
        numerator: 1,
        denominator: NonZeroI32::new(1).unwrap(),
    };

    pub const fn new(numerator: i32, denominator: i32) -> Option<Self> {
        match NonZeroI32::new(denominator) {
            None => None,
            Some(non_zero) => Some(Self {
                numerator,
                denominator: non_zero,
            }),
        }
    }

    pub const fn from_backend(value: AVRational) -> Option<Self> {
        Self::new(value.num, value.den)
    }

    pub const fn to_backend(self) -> AVRational {
        AVRational {
            num: self.numerator,
            den: self.denominator.get(),
        }
    }

    pub const fn to_f32(self) -> f32 {
        self.numerator as f32 / self.denominator.get() as f32
    }

    pub const fn to_f64(self) -> f64 {
        self.numerator as f64 / self.denominator.get() as f64
    }

    pub const fn inv(self) -> Option<Self> {
        Self::new(self.denominator.get(), self.numerator)
    }

    pub const fn is_zero(self) -> bool {
        self.numerator == 0
    }
}

impl Default for RatioI32 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Debug for RatioI32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

#[repr(C)]
pub struct CodecParameters(AVCodecParameters);

impl CodecParameters {
    /// General type of the encoded data.
    pub const fn media_type(&self) -> Option<MediaType> {
        MediaType::from_backend(self.0.codec_type)
    }

    /// Specific type of the encoded data (the codec used).
    pub const fn codec_id(&self) -> CodecId {
        CodecId(self.0.codec_id)
    }

    /// Additional information about the codec (corresponds to the AVI FOURCC).
    pub const fn codec_tag(&self) -> u32 {
        self.0.codec_tag
    }

    pub const fn format(&self) -> Option<AudioVideoFormat> {
        match self.media_type() {
            None | Some(MediaType::Attachment | MediaType::Subtitle | MediaType::Data) => None,
            Some(MediaType::Video) => AudioVideoFormat::video_from_i32(self.0.format),
            Some(MediaType::Audio) => AudioVideoFormat::audio_from_i32(self.0.format),
        }
    }

    /// The average bitrate of the encoded data (in bits per second).
    pub const fn bit_rate(&self) -> i64 {
        self.0.bit_rate
    }

    /// The number of bits per sample in the codedwords.
    ///
    /// This is basically the bitrate per sample. It is mandatory for a bunch of
    /// formats to actually decode them. It's the number of bits for one sample in
    /// the actual coded bitstream.
    ///
    /// This could be for example 4 for ADPCM
    /// For PCM formats this matches bits_per_raw_sample
    /// Can be 0
    pub const fn bits_per_coded_sample(&self) -> i32 {
        self.0.bits_per_coded_sample
    }

    /// This is the number of valid bits in each output sample. If the
    /// sample format has more bits, the least significant bits are additional
    /// padding bits, which are always 0. Use right shifts to reduce the sample
    /// to its actual size. For example, audio formats with 24 bit samples will
    /// have bits_per_raw_sample set to 24, and format set to AV_SAMPLE_FMT_S32.
    /// To get the original sample use "(int32_t)sample >> 8"."
    ///
    /// For ADPCM this might be 12 or 16 or similar
    /// Can be 0
    pub const fn bits_per_raw_sample(&self) -> i32 {
        self.0.bits_per_raw_sample
    }

    /// Video only. The width of the video frame in pixels.
    pub const fn video_width(&self) -> Option<u32> {
        match (self.media_type(), self.0.width) {
            (Some(MediaType::Video), width @ 0..) => Some(width.cast_unsigned()),
            _ => None,
        }
    }

    /// Video only. The height of the video frame in pixels.
    pub const fn video_height(&self) -> Option<u32> {
        match (self.media_type(), self.0.height) {
            (Some(MediaType::Video), height @ 0..) => Some(height.cast_unsigned()),
            _ => None,
        }
    }

    /// Video only. The dimensions of the video frame in pixels.
    pub const fn video_size(&self) -> Option<UVec2> {
        match (self.media_type(), self.0.width, self.0.height) {
            (Some(MediaType::Video), width @ 0.., height @ 0..) => {
                Some(UVec2::new(width.cast_unsigned(), height.cast_unsigned()))
            }
            _ => None,
        }
    }

    /// Video only. The aspect ratio (width / height) which a single pixel
    /// should have when displayed.
    ///
    /// When the aspect ratio is unknown / undefined, the numerator should be
    /// set to 0 (the denominator may have any value).
    pub const fn sample_aspect_ratio(&self) -> Option<RatioI32> {
        RatioI32::from_backend(self.0.sample_aspect_ratio)
    }

    /// Video only. Number of frames per second, for streams with constant frame
    /// durations. Should be set to { 0, 1 } when some frames have differing
    /// durations or if the value is not known.
    ///
    /// # Note
    ///
    /// This field correponds to values that are stored in codec-level
    /// headers and is typically overridden by container/transport-layer
    /// timestamps, when available. It should thus be used only as a last resort,
    /// when no higher-level timing information is available.
    pub const fn frame_rate(&self) -> Option<RatioI32> {
        RatioI32::from_backend(self.0.framerate)
    }
}

impl fmt::Debug for CodecParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CodecParameters")
            .field("media_type", &self.media_type())
            .field("codec_id", &self.codec_id())
            .field("format", &self.format())
            .field("bit_rate", &self.bit_rate())
            .field("bits_per_coded_sample", &self.bits_per_coded_sample())
            .field("bits_per_raw_sample", &self.bits_per_raw_sample())
            .field("video_size", &self.video_size())
            .field("sample_aspect_ratio", &self.sample_aspect_ratio())
            .field("frame_rate", &self.frame_rate())
            .finish_non_exhaustive()
    }
}

pub struct CodecContext {
    raw: NonNull<AVCodecContext>,
}

implement_raw!(CodecContext: AVCodecContext);

impl CodecContext {
    pub fn from_parameters(parameters: &CodecParameters) -> Result<Self, BackendError> {
        // FIXME(hack3rmann): may result in suboptimal behavior ((C) docs)
        let Some(codec_context_ptr) = NonNull::new(unsafe { avcodec_alloc_context3(ptr::null()) })
        else {
            panic!("unexpected libav error");
        };

        BackendError::result_of(unsafe {
            avcodec_parameters_to_context(
                codec_context_ptr.as_ptr(),
                (&raw const *parameters).cast(),
            )
        })?;

        Ok(Self {
            raw: codec_context_ptr,
        })
    }

    pub fn from_stream(stream: &Stream) -> Result<Self, BackendError> {
        Self::from_parameters(stream.codec_parameters())
    }

    pub fn send_packet(&mut self, packet: &Packet) -> Result<(), BackendError> {
        BackendError::result_of(unsafe {
            avcodec_send_packet(self.as_raw().as_ptr(), packet.as_raw().as_ptr())
        })
    }

    pub const fn codec_id(&self) -> CodecId {
        CodecId(unsafe { (*self.as_raw().as_ptr()).codec_id })
    }

    pub const fn codec(&self) -> Option<&Codec> {
        match NonNull::new(unsafe { (*self.as_raw().as_ptr()).codec }.cast_mut()) {
            None => None,
            Some(non_null) => Some(unsafe { non_null.cast().as_ref() }),
        }
    }

    pub fn open(&mut self, codec: &Codec) -> Result<(), BackendError> {
        BackendError::result_of(unsafe {
            avcodec_open2(
                self.as_raw().as_ptr(),
                (&raw const *codec).cast(),
                ptr::null_mut(),
            )
        })
    }

    pub fn receive_frame(&mut self, frame: &mut Frame) -> Result<(), BackendError> {
        BackendError::result_of(unsafe {
            avcodec_receive_frame(self.as_raw().as_ptr(), frame.as_raw().as_ptr())
        })
    }
}

pub struct Frame {
    raw: NonNull<AVFrame>,
}

implement_raw!(Frame: AVFrame);

impl Frame {
    pub fn new() -> Self {
        let frame_ptr = unsafe { av_frame_alloc() };
        Self {
            raw: NonNull::new(frame_ptr).expect("av_frame_alloc() failed"),
        }
    }

    pub const fn width(&self) -> u32 {
        match unsafe { (*self.as_raw().as_ptr()).width } {
            ..0 => unsafe { hint::unreachable_unchecked() },
            non_negative @ 0.. => non_negative.cast_unsigned(),
        }
    }

    pub const fn height(&self) -> u32 {
        match unsafe { (*self.as_raw().as_ptr()).height } {
            ..0 => unsafe { hint::unreachable_unchecked() },
            non_negative @ 0.. => non_negative.cast_unsigned(),
        }
    }

    pub const fn size(&self) -> UVec2 {
        UVec2::new(self.width(), self.height())
    }

    pub const fn format(&self) -> Option<VideoPixelFormat> {
        VideoPixelFormat::from_i32(unsafe { (*self.as_raw().as_ptr()).format })
    }

    /// # Safety
    ///
    /// FIXME(hack3rmann): unsafe, should rewrite
    pub unsafe fn plane_height(&self, index: usize) -> u32 {
        if index != 1 && index != 2 {
            return self.height();
        }

        let Some(desc) = self.format().unwrap().descriptor() else {
            return self.height();
        };

        let s = desc.log2_chroma_h();
        (self.height() + (1 << s) - 1) >> s
    }

    /// # Safety
    ///
    /// FIXME(hack3rmann): unsafe, should rewrite
    pub unsafe fn data(&self, index: usize) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                (*self.as_raw().as_ptr()).data[index],
                (*self.as_raw().as_ptr()).linesize[index] as usize
                    * self.plane_height(index) as usize,
            )
        }
    }

    /// # Safety
    ///
    /// FIXME(hack3rmann): unsafe, should rewrite
    pub unsafe fn is_empty(&self) -> bool {
        unsafe { (*self.as_raw().as_ptr()).data[0] }.is_null()
    }

    /// # Safety
    ///
    /// FIXME(hack3rmann): unsafe, should rewrite
    pub unsafe fn alloc(
        &mut self,
        format: VideoPixelFormat,
        size: UVec2,
    ) -> Result<(), BackendError> {
        let this = self.as_raw().as_ptr();
        unsafe { (*this).format = format as i32 };
        unsafe { (*this).width = size.x as i32 };
        unsafe { (*this).height = size.y as i32 };

        // FIXME(hack3rmann): possible memory leak
        BackendError::result_of(unsafe { av_frame_get_buffer(self.as_raw().as_ptr(), 32) })
    }
}

impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Frame")
            .field("size", &self.size())
            .finish_non_exhaustive()
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        let mut ptr = self.raw.as_ptr();
        unsafe { av_frame_free(&raw mut ptr) };
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Packet {
    raw: NonNull<AVPacket>,
}

implement_raw!(Packet: AVPacket);

impl Packet {
    pub fn new() -> Self {
        Self {
            raw: NonNull::new(unsafe { av_packet_alloc() }).expect("av_packet_alloc() failed"),
        }
    }

    pub fn init(&mut self, size: usize) -> Result<(), BackendError> {
        BackendError::result_of(unsafe {
            av_new_packet(self.as_raw().as_ptr(), size.try_into().unwrap())
        })
    }

    pub fn with_capacity(size: usize) -> Result<Self, BackendError> {
        let mut result = Self::new();
        result.init(size)?;
        Ok(result)
    }

    pub fn try_clone(&self) -> Result<Self, BackendError> {
        let packet_ptr = Self::new();
        BackendError::result_of(unsafe {
            av_packet_ref(
                packet_ptr.as_raw().as_ptr(),
                self.as_raw().as_ptr().cast_const(),
            )
        })
        .map(|()| packet_ptr)
    }

    pub const fn stream_index(&self) -> usize {
        match unsafe { (*self.as_raw().as_ptr()).stream_index } {
            ..0 => unsafe { hint::unreachable_unchecked() },
            non_negative @ 0.. => non_negative as usize,
        }
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Packet")
            .field("stream_index", &self.stream_index())
            .finish_non_exhaustive()
    }
}

impl Clone for Packet {
    fn clone(&self) -> Self {
        self.try_clone().unwrap()
    }
}

impl Drop for Packet {
    fn drop(&mut self) {
        let mut ptr = self.as_raw().as_ptr();
        // FIXME(hack3rmann): use free on owned packets only
        unsafe { av_packet_unref(ptr) };
        unsafe { av_packet_free(&raw mut ptr) };
    }
}

impl Default for Packet {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Profile(AVProfile);

impl Profile {
    pub const fn id(self) -> i32 {
        self.0.profile
    }

    pub const fn name(self) -> &'static str {
        let name_cstr = unsafe { CStr::from_ptr(self.0.name) };
        unsafe { str::from_utf8_unchecked(name_cstr.to_bytes()) }
    }
}

impl fmt::Debug for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Profile")
            .field("id", &self.id())
            .field("name", &self.name())
            .finish()
    }
}

#[repr(transparent)]
pub struct Codec(AVCodec);

impl Codec {
    pub fn find_for_id(id: CodecId) -> Option<&'static Self> {
        unsafe { avcodec_find_decoder(id.0).cast::<Self>().as_ref() }
    }

    pub const fn name(&self) -> &str {
        let name_cstr = unsafe { CStr::from_ptr(self.0.name) };
        unsafe { str::from_utf8_unchecked(name_cstr.to_bytes()) }
    }

    pub const fn long_name(&self) -> &str {
        let name_cstr = unsafe { CStr::from_ptr(self.0.long_name) };
        unsafe { str::from_utf8_unchecked(name_cstr.to_bytes()) }
    }

    pub const fn media_type(&self) -> Option<MediaType> {
        MediaType::from_backend(self.0.type_)
    }

    pub const fn id(&self) -> CodecId {
        CodecId(self.0.id)
    }

    pub const fn profiles(&self) -> Option<&[Profile]> {
        if self.0.profiles.is_null() {
            return None;
        }

        let mut profile_ptr = self.0.profiles;
        let mut count = 0;

        while unsafe { (*profile_ptr).profile } != AV_PROFILE_UNKNOWN {
            profile_ptr = profile_ptr.wrapping_add(1);
            count += 1;
        }

        Some(unsafe { slice::from_raw_parts(self.0.profiles.cast(), count) })
    }

    pub const fn profile_iterator(&self) -> Option<ProfileIterator<'_>> {
        match NonNull::new(self.0.profiles.cast_mut()) {
            None => None,
            Some(non_null) => Some(unsafe { ProfileIterator::from_raw(non_null) }),
        }
    }

    pub const fn wrapper_name(&self) -> Option<&str> {
        if self.0.wrapper_name.is_null() {
            return None;
        }

        let wrapper_name_cstr = unsafe { CStr::from_ptr(self.0.wrapper_name) };
        Some(unsafe { str::from_utf8_unchecked(wrapper_name_cstr.to_bytes()) })
    }

    pub fn is_decoder(&self) -> bool {
        0 != unsafe { av_codec_is_decoder(&raw const self.0) }
    }

    pub fn is_encoder(&self) -> bool {
        0 != unsafe { av_codec_is_encoder(&raw const self.0) }
    }
}

impl fmt::Debug for Codec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Codec")
            .field("name", &self.name())
            .field("long_name", &self.long_name())
            .field("media_type", &self.media_type())
            .field("id", &self.id())
            .field("wrapper_name", &self.wrapper_name())
            .finish_non_exhaustive()
    }
}

pub struct ProfileIterator<'s> {
    ptr: NonNull<AVProfile>,
    _p: PhantomData<&'s Codec>,
}

impl<'s> ProfileIterator<'s> {
    /// # Safety
    /// FIXME(hack3rmann): safety
    pub const unsafe fn from_raw(ptr: NonNull<AVProfile>) -> Self {
        Self {
            ptr,
            _p: PhantomData,
        }
    }
}

impl<'s> Iterator for ProfileIterator<'s> {
    type Item = &'s Profile;

    fn next(&mut self) -> Option<Self::Item> {
        if unsafe { (*self.ptr.as_ptr()).profile } == AV_PROFILE_UNKNOWN {
            return None;
        }

        let value = unsafe { self.ptr.cast::<Profile>().as_ref() };
        self.ptr = unsafe { self.ptr.add(1) };

        Some(value)
    }
}

pub struct CodecIterator {
    raw: *mut c_void,
}

impl CodecIterator {
    pub const fn new() -> Self {
        CodecIterator {
            raw: ptr::null_mut(),
        }
    }
}

impl Default for CodecIterator {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for CodecIterator {
    type Item = &'static Codec;

    fn next(&mut self) -> Option<Self::Item> {
        let codec = unsafe { av_codec_iterate(&raw mut self.raw) };
        let ptr = NonNull::new(codec.cast_mut())?.cast::<Codec>();
        Some(unsafe { ptr.as_ref() })
    }
}

pub struct ScalerFormat {
    pub size: UVec2,
    pub format: VideoPixelFormat,
}

bitflags! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct ScalerFlags: i32 {
        const FAST_BILINEAR = SWS_FAST_BILINEAR;
        const BILINEAR = SWS_BILINEAR;
        const BICUBIC = SWS_BICUBIC;
        const X = SWS_X;
        const POINT = SWS_POINT;
        const AREA = SWS_AREA;
        const BICUBLIN = SWS_BICUBLIN;
        const GAUSS = SWS_GAUSS;
        const SINC = SWS_SINC;
        const LANCZOS = SWS_LANCZOS;
        const SPLINE = SWS_SPLINE;
        const SRC_V_CHR_DROP_MASK = SWS_SRC_V_CHR_DROP_MASK;
        const SRC_V_CHR_DROP_SHIFT = SWS_SRC_V_CHR_DROP_SHIFT;
        const PARAM_DEFAULT = SWS_PARAM_DEFAULT;
        const PRINT_INFO = SWS_PRINT_INFO;
        const FULL_CHR_H_INT = SWS_FULL_CHR_H_INT;
        const FULL_CHR_H_INP = SWS_FULL_CHR_H_INP;
        const DIRECT_BGR = SWS_DIRECT_BGR;
        const ACCURATE_RND = SWS_ACCURATE_RND;
        const BITEXACT = SWS_BITEXACT;
        const ERROR_DIFFUSION = SWS_ERROR_DIFFUSION;
    }
}

pub struct Scaler {
    raw: NonNull<SwsContext>,
    source_format: ScalerFormat,
    destination_format: ScalerFormat,
}

impl Scaler {
    pub const fn as_raw(&self) -> NonNull<SwsContext> {
        self.raw
    }

    pub fn new(
        source_format: ScalerFormat,
        destination_format: ScalerFormat,
        flags: ScalerFlags,
    ) -> Result<Self, BackendError> {
        let ptr = unsafe {
            sws_getContext(
                source_format.size.x.cast_signed(),
                source_format.size.y.cast_signed(),
                source_format.format.to_backend(),
                destination_format.size.x.cast_signed(),
                destination_format.size.y.cast_signed(),
                destination_format.format.to_backend(),
                flags.bits(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null(),
            )
        };

        let Some(raw) = NonNull::new(ptr) else {
            return Err(BackendError::INVALID_DATA);
        };

        Ok(Self {
            raw,
            source_format,
            destination_format,
        })
    }

    pub fn run(&mut self, input: &Frame, output: &mut Frame) -> Result<(), BackendError> {
        if input.format() != Some(self.source_format.format)
            || input.size() != self.source_format.size
        {
            return Err(BackendError::INVALID_DATA);
        }

        // FIXME(hacl3rmann): reallocate better
        if unsafe { output.is_empty() } {
            unsafe { output.alloc(self.destination_format.format, self.destination_format.size) }?;
        }

        if output.format() != Some(self.destination_format.format)
            || output.size() != self.destination_format.size
        {
            return Err(BackendError::INVALID_DATA);
        }

        unsafe {
            sws_scale(
                self.as_raw().as_ptr(),
                (*input.as_raw().as_ptr()).data.as_ptr().cast(),
                (*input.as_raw().as_ptr()).linesize.as_ptr(),
                0,
                self.source_format.size.y as i32,
                (*output.as_raw().as_ptr()).data.as_ptr(),
                (*output.as_raw().as_ptr()).linesize.as_mut_ptr(),
            );
        }

        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct BackendError {
    // TODO(hack3rmann): use non-zero u31 for it
    encoded_code: NonZeroU32,
}

impl BackendError {
    pub const EOF: Self = Self::new(AVERROR_EOF).unwrap();
    pub const BSF_NOT_FOUND: Self = Self::new(AVERROR_BSF_NOT_FOUND).unwrap();
    pub const BUG: Self = Self::new(AVERROR_BUG).unwrap();
    pub const BUFFER_TOO_SMALL: Self = Self::new(AVERROR_BUFFER_TOO_SMALL).unwrap();
    pub const DECODER_NOT_FOUND: Self = Self::new(AVERROR_DECODER_NOT_FOUND).unwrap();
    pub const DEMUXER_NOT_FOUND: Self = Self::new(AVERROR_DEMUXER_NOT_FOUND).unwrap();
    pub const ENCODER_NOT_FOUND: Self = Self::new(AVERROR_ENCODER_NOT_FOUND).unwrap();
    pub const EXIT: Self = Self::new(AVERROR_EXIT).unwrap();
    pub const EXTERNAL: Self = Self::new(AVERROR_EXTERNAL).unwrap();
    pub const FILTER_NOT_FOUND: Self = Self::new(AVERROR_FILTER_NOT_FOUND).unwrap();
    pub const INVALID_DATA: Self = Self::new(AVERROR_INVALIDDATA).unwrap();
    pub const MUXER_NOT_FOUND: Self = Self::new(AVERROR_MUXER_NOT_FOUND).unwrap();
    pub const OPTION_NOT_FOUND: Self = Self::new(AVERROR_OPTION_NOT_FOUND).unwrap();
    pub const PATCH_WELCOME: Self = Self::new(AVERROR_PATCHWELCOME).unwrap();
    pub const PROTOCOL_NOT_FOUND: Self = Self::new(AVERROR_PROTOCOL_NOT_FOUND).unwrap();
    pub const STREAM_NOT_FOUND: Self = Self::new(AVERROR_STREAM_NOT_FOUND).unwrap();
    pub const BUG2: Self = Self::new(AVERROR_BUG2).unwrap();
    pub const UNKNOWN: Self = Self::new(AVERROR_UNKNOWN).unwrap();
    pub const HTTP_BAD_REQUEST: Self = Self::new(AVERROR_HTTP_BAD_REQUEST).unwrap();
    pub const HTTP_UNAUTHORIZED: Self = Self::new(AVERROR_HTTP_UNAUTHORIZED).unwrap();
    pub const HTTP_FORBIDDEN: Self = Self::new(AVERROR_HTTP_FORBIDDEN).unwrap();
    pub const HTTP_NOT_FOUND: Self = Self::new(AVERROR_HTTP_NOT_FOUND).unwrap();
    pub const HTTP_TOO_MANY_REQUESTS: Self = Self::new(AVERROR_HTTP_TOO_MANY_REQUESTS).unwrap();
    pub const HTTP_OTHER_4XX: Self = Self::new(AVERROR_HTTP_OTHER_4XX).unwrap();
    pub const HTTP_SERVER_ERROR: Self = Self::new(AVERROR_HTTP_SERVER_ERROR).unwrap();

    pub const fn result_of(code: i32) -> Result<(), Self> {
        match Self::new(code) {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    pub const fn result_or_non_negative(code: i32) -> Result<u32, Self> {
        match Self::new(code) {
            Some(error) => Err(error),
            None => Ok(code.cast_unsigned()),
        }
    }

    pub const fn new(code: i32) -> Option<Self> {
        match code {
            ..0 => Some(Self {
                encoded_code: unsafe {
                    NonZeroU32::new_unchecked(code.wrapping_neg().cast_unsigned())
                },
            }),
            0.. => None,
        }
    }

    pub const fn code(self) -> i32 {
        self.encoded_code.get().cast_signed().wrapping_neg()
    }
}

impl fmt::Debug for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.code(), f)
    }
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // FIXME(hack3rmann): get the error string from libav
        write!(f, "BackendError({})", self.code())
    }
}

impl Error for BackendError {
    fn description(&self) -> &str {
        "libav backend error"
    }
}
