pub mod format;

use bitflags::bitflags;
use ffmpeg_sys_next::{
    AV_PROFILE_UNKNOWN, AV_TIME_BASE_Q, AVCodec, AVCodecContext, AVCodecID, AVCodecParameters,
    AVDiscard, AVERROR_BSF_NOT_FOUND, AVERROR_BUFFER_TOO_SMALL, AVERROR_BUG, AVERROR_BUG2,
    AVERROR_DECODER_NOT_FOUND, AVERROR_DEMUXER_NOT_FOUND, AVERROR_ENCODER_NOT_FOUND, AVERROR_EOF,
    AVERROR_EXIT, AVERROR_EXTERNAL, AVERROR_FILTER_NOT_FOUND, AVERROR_HTTP_BAD_REQUEST,
    AVERROR_HTTP_FORBIDDEN, AVERROR_HTTP_NOT_FOUND, AVERROR_HTTP_OTHER_4XX,
    AVERROR_HTTP_SERVER_ERROR, AVERROR_HTTP_TOO_MANY_REQUESTS, AVERROR_HTTP_UNAUTHORIZED,
    AVERROR_INVALIDDATA, AVERROR_MUXER_NOT_FOUND, AVERROR_OPTION_NOT_FOUND, AVERROR_PATCHWELCOME,
    AVERROR_PROTOCOL_NOT_FOUND, AVERROR_STREAM_NOT_FOUND, AVERROR_UNKNOWN, AVFormatContext,
    AVFrame, AVHWDeviceType, AVMediaType, AVPacket, AVPixFmtDescriptor, AVProfile, AVRational,
    AVStream, SEEK_SET, SWS_ACCURATE_RND, SWS_AREA, SWS_BICUBIC, SWS_BICUBLIN, SWS_BILINEAR,
    SWS_BITEXACT, SWS_DIRECT_BGR, SWS_ERROR_DIFFUSION, SWS_FAST_BILINEAR, SWS_FULL_CHR_H_INP,
    SWS_FULL_CHR_H_INT, SWS_GAUSS, SWS_LANCZOS, SWS_PARAM_DEFAULT, SWS_POINT, SWS_PRINT_INFO,
    SWS_SINC, SWS_SPLINE, SWS_SRC_V_CHR_DROP_MASK, SWS_SRC_V_CHR_DROP_SHIFT, SWS_X, SwsContext,
    av_codec_is_decoder, av_codec_is_encoder, av_codec_iterate, av_find_best_stream,
    av_frame_alloc, av_frame_free, av_frame_get_buffer, av_hwdevice_get_type_name,
    av_hwdevice_iterate_types, av_new_packet, av_packet_alloc, av_packet_free, av_packet_ref,
    av_packet_unref, av_read_frame, av_strerror, avcodec_alloc_context3, avcodec_find_decoder,
    avcodec_open2, avcodec_parameters_copy, avcodec_parameters_to_context, avcodec_receive_frame,
    avcodec_send_packet, avdevice_register_all, avformat_close_input, avformat_find_stream_info,
    avformat_open_input, avformat_seek_file, avio_seek, strerror, sws_getContext, sws_scale,
};
use glam::UVec2;
use std::{
    error::Error,
    ffi::{CStr, c_void},
    fmt, hint,
    marker::PhantomData,
    num::{NonZeroI32, NonZeroI64, NonZeroU64},
    ptr::{self, NonNull},
    slice, str,
    time::Duration,
};

pub use ffmpeg_sys_next::AVComponentDescriptor as ComponentDescriptor;
pub use format::{AudioVideoFormat, PixelFormatFlags, VideoPixelFormat};

/// Initialize `libavdevice` and register all the input and output devices.
pub fn init() {
    unsafe { avdevice_register_all() };
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
pub enum HwDeviceType {
    VdPau = 1,
    Cuda = 2,
    #[default]
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
    /// Get FFI-compatible value
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

    /// Construct [`HwDeviceType`] from FFI-compatible value
    ///
    /// # Note
    ///
    /// Returns [`None`] if `value` is [`AVHWDeviceType::AV_HWDEVICE_TYPE_NONE`]
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

    /// Get the string name of an [`HwDeviceType`]
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

/// Iterate over supported device types.
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
            #[allow(clippy::missing_safety_doc)]
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

/// Media type
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

    /// Get FFI-compatible value
    pub const fn to_backend(self) -> AVMediaType {
        match self {
            MediaType::Video => AVMediaType::AVMEDIA_TYPE_VIDEO,
            MediaType::Audio => AVMediaType::AVMEDIA_TYPE_AUDIO,
            MediaType::Data => AVMediaType::AVMEDIA_TYPE_DATA,
            MediaType::Subtitle => AVMediaType::AVMEDIA_TYPE_SUBTITLE,
            MediaType::Attachment => AVMediaType::AVMEDIA_TYPE_ATTACHMENT,
        }
    }

    /// Construct [`MediaType`] from FFI-compatible value
    ///
    /// # Note
    ///
    /// Returns [`None`] if `value` is either [`AVMediaType::AVMEDIA_TYPE_UNKNOWN`] or
    /// [`AVMediaType::AVMEDIA_TYPE_NB`]
    pub const fn from_backend(value: AVMediaType) -> Option<Self> {
        Some(match value {
            AVMediaType::AVMEDIA_TYPE_VIDEO => Self::Video,
            AVMediaType::AVMEDIA_TYPE_AUDIO => Self::Audio,
            AVMediaType::AVMEDIA_TYPE_DATA => Self::Data,
            AVMediaType::AVMEDIA_TYPE_SUBTITLE => Self::Subtitle,
            AVMediaType::AVMEDIA_TYPE_ATTACHMENT => Self::Attachment,
            AVMediaType::AVMEDIA_TYPE_NB | AVMediaType::AVMEDIA_TYPE_UNKNOWN => return None,
        })
    }
}

/// Format I/O context.
pub struct FormatContext {
    raw: NonNull<AVFormatContext>,
}

implement_raw!(FormatContext: AVFormatContext);

impl FormatContext {
    /// Open an input stream and read the header. The codecs are not opened.
    ///
    /// # Parameters
    ///
    /// - `url` - URL of the stream to open.
    ///
    /// # Note
    ///
    /// If you want to use custom IO, preallocate the format context and set its pb field.
    pub fn from_input(url: &CStr) -> Result<Self, BackendError> {
        let mut context_ptr = ptr::null_mut();

        BackendError::result_of(unsafe {
            avformat_open_input(
                &raw mut context_ptr,
                url.as_ptr(),
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
        unsafe { slice::from_raw_parts(ptr.cast(), len as usize) }
    }

    pub const fn streams_mut(&mut self) -> &mut [Stream] {
        let ptr = unsafe { (*self.as_raw().as_ptr()).streams };
        let len = unsafe { (*self.as_raw().as_ptr()).nb_streams };
        unsafe { slice::from_raw_parts_mut(ptr.cast(), len as usize) }
    }

    pub fn find_best_stream(&self, media_type: MediaType) -> Result<&Stream, BackendError> {
        let index = match BackendError::result_or_u32(unsafe {
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

    pub fn read_packet(&mut self) -> Result<Packet, BackendError> {
        let packet = Packet::new();
        BackendError::result_of(unsafe {
            av_read_frame(self.as_raw().as_ptr(), packet.as_raw().as_ptr())
        })
        .map(|()| packet)
    }

    pub fn repeat_stream(&mut self, index: usize) -> Result<(), BackendError> {
        let io_context_ptr = unsafe { (*self.as_raw().as_ptr()).pb };
        let _new_pos =
            BackendError::result_or_u64(unsafe { avio_seek(io_context_ptr, 0, SEEK_SET) })?;

        let stream = &self.streams()[index];
        let duration = unsafe { (*stream.as_raw().as_ptr()).duration };

        BackendError::result_of(unsafe {
            avformat_seek_file(self.as_raw().as_ptr(), index as i32, 0, 0, duration, 0)
        })
    }
}

impl Drop for FormatContext {
    fn drop(&mut self) {
        let mut ptr = self.raw.as_ptr();
        unsafe { avformat_close_input(&raw mut ptr) };
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
    pub struct DispositionFlags: i32 {
        const DEFAULT = 1;
        const DUB = 2;
        const ORIGINAL = 4;
        const COMMENT = 8;
        const LYRICS = 16;
        const KARAOKE = 32;
        const FORCED = 64;
        const HEARING_IMPAIRED = 128;
        const VISUAL_IMPAIRED = 256;
        const CLEAN_EFFECTS = 512;
        const ATTACHED_PIC = 1024;
        const TIMED_THUMBNAILS = 2048;
        const NON_DIEGETIC = 4096;
        const CAPTIONS = 65536;
        const DESCRIPTIONS = 131072;
        const METADATA = 262144;
        const DEPENDENT = 524288;
        const STILL_IMAGE = 1048576;
        const MULTILAYER = 2097152;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Default, Hash)]
pub enum Discard {
    /// Discard nothing
    None = -16,
    /// Discard useless packets like 0 size packets in avi
    #[default]
    Default = 0,
    /// Discard all non reference
    NonReference = 8,
    /// Discard all bidirectional frames
    Bidirectional = 16,
    /// Discard all non intra frames
    NonIntra = 24,
    /// Discard all frames except keyframes
    NonKey = 32,
    /// Discard all
    All = 48,
}

impl Discard {
    pub const fn from_i32(value: i32) -> Option<Self> {
        Some(match value {
            -16 => Self::None,
            0 => Self::Default,
            8 => Self::NonReference,
            16 => Self::Bidirectional,
            24 => Self::NonIntra,
            32 => Self::NonKey,
            48 => Self::All,
            _ => return None,
        })
    }

    pub const fn from_backend(value: AVDiscard) -> Self {
        match value {
            AVDiscard::AVDISCARD_NONE => Self::None,
            AVDiscard::AVDISCARD_DEFAULT => Self::Default,
            AVDiscard::AVDISCARD_NONREF => Self::NonReference,
            AVDiscard::AVDISCARD_BIDIR => Self::Bidirectional,
            AVDiscard::AVDISCARD_NONINTRA => Self::NonIntra,
            AVDiscard::AVDISCARD_NONKEY => Self::NonKey,
            AVDiscard::AVDISCARD_ALL => Self::All,
        }
    }

    pub const fn to_backend(self) -> AVDiscard {
        match self {
            Self::None => AVDiscard::AVDISCARD_NONE,
            Self::Default => AVDiscard::AVDISCARD_DEFAULT,
            Self::NonReference => AVDiscard::AVDISCARD_NONREF,
            Self::Bidirectional => AVDiscard::AVDISCARD_BIDIR,
            Self::NonIntra => AVDiscard::AVDISCARD_NONINTRA,
            Self::NonKey => AVDiscard::AVDISCARD_NONKEY,
            Self::All => AVDiscard::AVDISCARD_ALL,
        }
    }
}

#[repr(C)]
pub struct Stream {
    raw: NonNull<AVStream>,
}

implement_raw!(Stream: AVStream);

impl Stream {
    /// Stream index in [`FormatContext`]
    pub const fn index(&self) -> usize {
        match unsafe { (*self.as_raw().as_ptr()).index } {
            ..0 => unsafe { hint::unreachable_unchecked() },
            non_negative @ 0.. => non_negative as usize,
        }
    }

    /// Format-specific stream ID.
    pub const fn id(&self) -> i32 {
        unsafe { (*self.as_raw().as_ptr()).id }
    }

    /// Duration of the stream, in stream time base.
    /// If a source file does not specify a duration, but does specify
    /// a bitrate, this value will be estimated from bitrate and file size.
    pub const fn duration(&self) -> Option<FrameDuration> {
        let duration_backend = unsafe { (*self.as_raw().as_ptr()).duration };
        let Some(duration) = NonZeroI64::new(duration_backend) else {
            return None;
        };

        let base_backend = unsafe { (*self.as_raw().as_ptr()).time_base };
        // Safety: 'set by libavformat' therefore has non-zero denominator
        let base = unsafe { RatioI32::from_backend(base_backend).unwrap_unchecked() };

        Some(FrameDuration { base, duration })
    }

    /// Number of frames in this stream if known
    pub const fn frame_count(&self) -> Option<NonZeroU64> {
        match unsafe { (*self.as_raw().as_ptr()).nb_frames } {
            ..0 => unsafe { hint::unreachable_unchecked() },
            non_negative @ 0.. => NonZeroU64::new(non_negative as u64),
        }
    }

    /// Stream disposition
    pub const fn disposition(&self) -> DispositionFlags {
        let bits = unsafe { (*self.as_raw().as_ptr()).disposition };
        unsafe { DispositionFlags::from_bits(bits).unwrap_unchecked() }
    }

    /// Selects which packets can be discarded at will and do not need to be demuxed.
    pub const fn discard(&self) -> Discard {
        Discard::from_backend(unsafe { (*self.as_raw().as_ptr()).discard })
    }

    /// Sample aspect ratio (`0` if unknown)
    pub const fn sample_aspect_ratio(&self) -> RatioI32 {
        let value_backend = unsafe { (*self.as_raw().as_ptr()).sample_aspect_ratio };
        unsafe { RatioI32::from_backend(value_backend).unwrap_unchecked() }
    }

    /// Codec parameters associated with this stream.
    pub const fn codec_parameters(&self) -> &CodecParameters {
        // TODO(hack3rmann): figure out safety for this
        unsafe {
            (*self.as_raw().as_ptr())
                .codecpar
                .cast::<CodecParameters>()
                .as_ref()
                .unwrap_unchecked()
        }
    }

    /// This is the fundamental unit of time (in seconds) in terms
    /// of which frame timestamps are represented.
    pub const fn time_base(&self) -> RatioI32 {
        let backend_value = unsafe { (*self.as_raw().as_ptr()).time_base };
        // Safety: 'set by libavformat' therefore has non-zero denominator
        unsafe { RatioI32::from_backend(backend_value).unwrap_unchecked() }
    }
}

impl fmt::Debug for Stream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stream")
            .field("index", &self.index())
            .field("id", &self.id())
            .field("duration", &self.duration())
            .field("frame_count", &self.frame_count())
            .field("codec_parameters", &self.codec_parameters())
            .field("time_base", &self.time_base())
            .field("disposition", &self.disposition())
            .field("discard", &self.discard())
            .field("sample_aspect_ratio", &self.sample_aspect_ratio())
            .finish_non_exhaustive()
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

    pub const fn to_duration_seconds(self) -> Duration {
        // HACK(hack3rmann): may overflow a lot
        let n_seconds = self.numerator as i64 / self.denominator.get() as i64;
        let n_nanoseconds =
            1_000_000_000_i64 * self.numerator as i64 / self.denominator.get() as i64;
        Duration::new(n_seconds.cast_unsigned(), n_nanoseconds as u32)
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
        match self.media_type() {
            Some(MediaType::Video) => Some(self.0.width.cast_unsigned()),
            _ => None,
        }
    }

    /// Video only. The height of the video frame in pixels.
    pub const fn video_height(&self) -> Option<u32> {
        match self.media_type() {
            Some(MediaType::Video) => Some(self.0.height.cast_unsigned()),
            _ => None,
        }
    }

    /// Video only. The dimensions of the video frame in pixels.
    pub const fn video_size(&self) -> Option<UVec2> {
        match self.media_type() {
            Some(MediaType::Video) => Some(UVec2::new(
                self.0.width.cast_unsigned(),
                self.0.height.cast_unsigned(),
            )),
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

    pub fn try_clone_from(&mut self, source: &Self) -> Result<(), BackendError> {
        BackendError::result_of(unsafe {
            avcodec_parameters_copy((&raw mut *self).cast(), (&raw const *source).cast())
        })
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

#[derive(Clone, Copy, Debug)]
pub struct FrameDuration {
    pub base: RatioI32,
    pub duration: NonZeroI64,
}

impl FrameDuration {
    pub const FALLBACK_BASE: RatioI32 = RatioI32::from_backend(AV_TIME_BASE_Q).unwrap();

    pub const fn to_duration(self) -> Duration {
        let n_seconds =
            self.duration.get() * self.base.numerator as i64 / self.base.denominator.get() as i64;
        let n_nanoseconds = 1_000_000_000_i64 * self.duration.get() * self.base.numerator as i64
            / self.base.denominator.get() as i64;
        Duration::new(n_seconds as u64, n_nanoseconds as u32)
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

    pub const fn duration_in(&self, base: RatioI32) -> Option<FrameDuration> {
        let Some(duration) = NonZeroI64::new(unsafe { (*self.as_raw().as_ptr()).duration }) else {
            return None;
        };

        Some(FrameDuration { base, duration })
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
    encoded_code: NonZeroI32,
}

impl BackendError {
    /// End of file
    pub const EOF: Self = Self::new(AVERROR_EOF).unwrap();
    /// Bitstream filter not found
    pub const BSF_NOT_FOUND: Self = Self::new(AVERROR_BSF_NOT_FOUND).unwrap();
    /// Internal bug, should not have happened
    pub const BUG: Self = Self::new(AVERROR_BUG).unwrap();
    /// Buffer too small
    pub const BUFFER_TOO_SMALL: Self = Self::new(AVERROR_BUFFER_TOO_SMALL).unwrap();
    /// Decoder not found
    pub const DECODER_NOT_FOUND: Self = Self::new(AVERROR_DECODER_NOT_FOUND).unwrap();
    /// Demuxer not found
    pub const DEMUXER_NOT_FOUND: Self = Self::new(AVERROR_DEMUXER_NOT_FOUND).unwrap();
    /// Encoder not found
    pub const ENCODER_NOT_FOUND: Self = Self::new(AVERROR_ENCODER_NOT_FOUND).unwrap();
    /// Immediate exit requested
    pub const EXIT: Self = Self::new(AVERROR_EXIT).unwrap();
    /// Generic error in an external library
    pub const EXTERNAL: Self = Self::new(AVERROR_EXTERNAL).unwrap();
    /// Filter not found
    pub const FILTER_NOT_FOUND: Self = Self::new(AVERROR_FILTER_NOT_FOUND).unwrap();
    /// Invalid data found when processing input
    pub const INVALID_DATA: Self = Self::new(AVERROR_INVALIDDATA).unwrap();
    /// Muxer not found
    pub const MUXER_NOT_FOUND: Self = Self::new(AVERROR_MUXER_NOT_FOUND).unwrap();
    /// Option not found
    pub const OPTION_NOT_FOUND: Self = Self::new(AVERROR_OPTION_NOT_FOUND).unwrap();
    /// Not yet implemented in FFmpeg, patches welcome
    pub const PATCH_WELCOME: Self = Self::new(AVERROR_PATCHWELCOME).unwrap();
    /// Protocol not found
    pub const PROTOCOL_NOT_FOUND: Self = Self::new(AVERROR_PROTOCOL_NOT_FOUND).unwrap();
    /// Stream not found
    pub const STREAM_NOT_FOUND: Self = Self::new(AVERROR_STREAM_NOT_FOUND).unwrap();
    /// Internal bug, should not have happened
    pub const BUG2: Self = Self::new(AVERROR_BUG2).unwrap();
    /// Unknown error occurred
    pub const UNKNOWN: Self = Self::new(AVERROR_UNKNOWN).unwrap();
    /// Server returned 400 Bad Request
    pub const HTTP_BAD_REQUEST: Self = Self::new(AVERROR_HTTP_BAD_REQUEST).unwrap();
    /// Server returned 401 Unauthorized (authorization failed)
    pub const HTTP_UNAUTHORIZED: Self = Self::new(AVERROR_HTTP_UNAUTHORIZED).unwrap();
    /// Server returned 403 Forbidden (access denied)
    pub const HTTP_FORBIDDEN: Self = Self::new(AVERROR_HTTP_FORBIDDEN).unwrap();
    /// Server returned 404 Not Found
    pub const HTTP_NOT_FOUND: Self = Self::new(AVERROR_HTTP_NOT_FOUND).unwrap();
    /// Server returned 429 Too Many Requests
    pub const HTTP_TOO_MANY_REQUESTS: Self = Self::new(AVERROR_HTTP_TOO_MANY_REQUESTS).unwrap();
    /// Server returned 4XX Client Error, but not one of 40{0,1,3,4}
    pub const HTTP_OTHER_4XX: Self = Self::new(AVERROR_HTTP_OTHER_4XX).unwrap();
    /// Server returned 5XX Server Error reply
    pub const HTTP_SERVER_ERROR: Self = Self::new(AVERROR_HTTP_SERVER_ERROR).unwrap();

    pub const ERROR_BUFFER_SIZE: usize = 128;

    pub const fn result_of(code: i32) -> Result<(), Self> {
        match Self::new(code) {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    pub const fn result_or_u32(code: i32) -> Result<u32, Self> {
        match Self::new(code) {
            Some(error) => Err(error),
            None => Ok(code.cast_unsigned()),
        }
    }

    pub const fn result_or_u64(code: i64) -> Result<u64, Self> {
        const I32_MIN: i64 = i32::MIN as i64;

        match code {
            error @ I32_MIN..0 => Err(unsafe { Self::new(error as i32).unwrap_unchecked() }),
            non_negative @ 0.. => Ok(non_negative.cast_unsigned()),
            ..I32_MIN => panic!("error code out of range"),
        }
    }

    pub const fn new(code: i32) -> Option<Self> {
        match code {
            ..0 => Some(Self {
                encoded_code: unsafe { NonZeroI32::new_unchecked(code) },
            }),
            0.. => None,
        }
    }

    pub const fn code(self) -> i32 {
        self.encoded_code.get()
    }

    fn posix_error_description(self) -> Option<&'static str> {
        let str_ptr = unsafe { strerror(self.code()) };

        if str_ptr.is_null() {
            return None;
        }

        let description_cstr = unsafe { CStr::from_ptr(str_ptr) };
        Some(unsafe { str::from_utf8_unchecked(description_cstr.to_bytes()) })
    }

    pub fn description(self) -> Option<String> {
        let mut buffer = Vec::<u8>::with_capacity(Self::ERROR_BUFFER_SIZE);

        match unsafe {
            av_strerror(
                self.code(),
                buffer.as_mut_ptr().cast(),
                Self::ERROR_BUFFER_SIZE,
            )
        } {
            1.. => return None,
            0 => {
                let cstr = unsafe { CStr::from_ptr(buffer.as_ptr().cast()) };
                unsafe { buffer.set_len(cstr.count_bytes()) };
            }
            ..0 => {
                let desc = self.posix_error_description()?;
                buffer.extend_from_slice(desc.as_bytes());
            }
        }

        Some(unsafe { String::from_utf8_unchecked(buffer) })
    }
}

impl fmt::Debug for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match BackendError::description(*self) {
            Some(desc) => write!(f, "BackendError({}): {desc}", self.code()),
            None => f.debug_tuple("BackendError").field(&self.code()).finish(),
        }
    }
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

impl Error for BackendError {
    fn description(&self) -> &str {
        "libav backend error"
    }
}
