pub mod acceleration;
pub mod codec;
pub mod error;
pub mod format;
pub mod hardware;
pub mod time;

use bitflags::bitflags;
use ffi::va;
use ffmpeg_sys_next::{
    AV_PROFILE_UNKNOWN, AVDiscard, AVFormatContext, AVFrame, AVMediaType, AVPacket,
    AVPixFmtDescriptor, AVProfile, AVStream, SEEK_SET, SWS_ACCURATE_RND, SWS_AREA, SWS_BICUBIC,
    SWS_BICUBLIN, SWS_BILINEAR, SWS_BITEXACT, SWS_DIRECT_BGR, SWS_ERROR_DIFFUSION,
    SWS_FAST_BILINEAR, SWS_FULL_CHR_H_INP, SWS_FULL_CHR_H_INT, SWS_GAUSS, SWS_LANCZOS,
    SWS_PARAM_DEFAULT, SWS_POINT, SWS_PRINT_INFO, SWS_SINC, SWS_SPLINE, SWS_SRC_V_CHR_DROP_MASK,
    SWS_SRC_V_CHR_DROP_SHIFT, SWS_X, SwsContext, av_buffer_get_ref_count, av_codec_iterate,
    av_find_best_stream, av_frame_alloc, av_frame_free, av_frame_get_buffer, av_frame_unref,
    av_new_packet, av_packet_alloc, av_packet_free, av_packet_ref, av_packet_unref, av_read_frame,
    avdevice_register_all, avformat_close_input, avformat_find_stream_info, avformat_open_input,
    avformat_seek_file, avio_seek, sws_getContext, sws_scale,
};
use glam::UVec2;
use std::{
    ffi::{CStr, c_void},
    fmt, hint,
    marker::PhantomData,
    num::{NonZeroI64, NonZeroU64},
    ptr::{self, NonNull},
    slice, str,
};

pub use acceleration::VaError;
pub use codec::{Codec, CodecContext, CodecId, CodecParameters, OwnedCodecParameters};
pub use error::BackendError;
pub use ffmpeg_sys_next::AVComponentDescriptor as ComponentDescriptor;
pub use format::{AudioVideoFormat, PixelFormatFlags, VideoPixelFormat};
pub use hardware::{HardwareDeviceType, HwDeviceTypeIterator};
pub use time::{FrameDuration, RatioI32};

pub mod ffi {
    pub use crate::acceleration::ffi as va;
    pub use ffmpeg_sys_next as ffmpeg;
}

/// Initialize `libavdevice` and register all the input and output devices.
pub fn init() {
    unsafe { avdevice_register_all() };
}

macro_rules! implement_raw {
    ( $Wrapper:ty { $raw:ident $( , $field:ident : $value:expr )* $(,)? } : $Raw:ty ) => {
        impl $Wrapper {
            #[allow(clippy::missing_safety_doc)]
            pub const unsafe fn from_raw(raw: ::std::ptr::NonNull<$Raw>) -> Self {
                Self {
                    $raw: raw,
                    $(
                        $field: $value,
                    )*
                }
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
        $crate::implement_raw!( $Wrapper { raw } : $Raw);
    };
}

pub(crate) use implement_raw;

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
    /// Number of enum values
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

unsafe impl Send for FormatContext {}
unsafe impl Sync for FormatContext {}

implement_raw!(FormatContext: AVFormatContext);

impl FormatContext {
    /// Open an input stream and read the header. The codecs are not opened.
    ///
    /// # Parameter
    ///
    /// `url` - URL of the stream to open.
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

    /// Read packets of a media file to get stream information. This
    /// is useful for file formats with no headers such as MPEG. This
    /// function also computes the real framerate in case of MPEG-2 repeat
    /// frame mode.
    /// The logical file position is not changed by this function;
    /// examined packets may be buffered for later processing.
    ///
    /// # Note
    ///
    /// This function isn't guaranteed to open all the codecs, so
    /// options being non-empty at return is a perfectly normal behavior.
    pub fn find_stream_info(&mut self) -> Result<(), BackendError> {
        BackendError::result_of(unsafe {
            avformat_find_stream_info(self.as_raw().as_ptr(), ptr::null_mut())
        })
    }

    /// A list of all streams in the file.
    pub const fn streams(&self) -> &[Stream] {
        let ptr = unsafe { (*self.as_raw().as_ptr()).streams };
        let len = unsafe { (*self.as_raw().as_ptr()).nb_streams };
        unsafe { slice::from_raw_parts(ptr.cast(), len as usize) }
    }

    /// A list of all streams in the file.
    pub const fn streams_mut(&mut self) -> &mut [Stream] {
        let ptr = unsafe { (*self.as_raw().as_ptr()).streams };
        let len = unsafe { (*self.as_raw().as_ptr()).nb_streams };
        unsafe { slice::from_raw_parts_mut(ptr.cast(), len as usize) }
    }

    /// Find the "best" stream in the file.
    /// The best stream is determined according to various heuristics as the most
    /// likely to be what the user expects.
    /// If the decoder parameter is non-NULL, av_find_best_stream will find the
    /// default decoder for the stream's codec; streams for which no decoder can
    /// be found are ignored.
    ///
    /// # Return
    ///
    /// - [`Ok`] in case of success
    /// - [`Err`] with [`BackendError::STREAM_NOT_FOUND`] if no stream with the
    ///   requested type could be found
    /// - [`Err`] witg [`BackendError::DECODER_NOT_FOUND`] if streams were found but not decoder
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

    /// Total stream bitrate in bit/s, [`None`] if not
    /// available. Never set it directly if the file_size and the
    /// duration are known as FFmpeg can compute it automatically.
    pub const fn bit_rate(&self) -> Option<NonZeroI64> {
        NonZeroI64::new(unsafe { (*self.as_raw().as_ptr()).bit_rate })
    }

    /// Return the next frame of a stream.
    /// This function returns what is stored in the file, and does not validate
    /// that what is there are valid frames for the decoder. It will split what is
    /// stored in the file into frames and return one for each call. It will not
    /// omit invalid data between valid frames so as to give the decoder the maximum
    /// information possible for decoding.
    ///
    /// On success, the returned packet is reference-counted (pkt->buf is set) and
    /// valid indefinitely. The packet must be freed with av_packet_unref() when
    /// it is no longer needed. For video, the packet contains exactly one frame.
    /// For audio, it contains an integer number of frames if each frame has
    /// a known fixed size (e.g. PCM or ADPCM data). If the audio frames have
    /// a variable size (e.g. MPEG audio), then it contains one frame.
    ///
    /// pkt->pts, pkt->dts and pkt->duration are always set to correct
    /// values in AVStream.time_base units (and guessed if the format cannot
    /// provide them). pkt->pts can be AV_NOPTS_VALUE if the video format
    /// has B-frames, so it is better to rely on pkt->dts if you do not
    /// decompress the payload.
    pub fn read_any_packet(&mut self) -> Result<Packet, BackendError> {
        let packet = Packet::new();
        BackendError::result_of(unsafe {
            av_read_frame(self.as_raw().as_ptr(), packet.as_raw().as_ptr())
        })
        .map(|()| packet)
    }

    /// Return the next frame of the concrete stream.
    /// This function returns what is stored in the file, and does not validate
    /// that what is there are valid frames for the decoder. It will split what is
    /// stored in the file into frames and return one for each call. It will not
    /// omit invalid data between valid frames so as to give the decoder the maximum
    /// information possible for decoding.
    ///
    /// On success, the returned packet is reference-counted (pkt->buf is set) and
    /// valid indefinitely. The packet must be freed with av_packet_unref() when
    /// it is no longer needed. For video, the packet contains exactly one frame.
    /// For audio, it contains an integer number of frames if each frame has
    /// a known fixed size (e.g. PCM or ADPCM data). If the audio frames have
    /// a variable size (e.g. MPEG audio), then it contains one frame.
    ///
    /// pkt->pts, pkt->dts and pkt->duration are always set to correct
    /// values in AVStream.time_base units (and guessed if the format cannot
    /// provide them). pkt->pts can be AV_NOPTS_VALUE if the video format
    /// has B-frames, so it is better to rely on pkt->dts if you do not
    /// decompress the payload.
    pub fn read_packet(&mut self, stream_index: usize) -> Result<Packet, BackendError> {
        Ok(loop {
            let packet = self.read_any_packet()?;

            if packet.stream_index() == stream_index {
                break packet;
            }
        })
    }

    /// Seeks to the start of the input file
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

impl fmt::Debug for FormatContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FormatContext")
            .field("bit_rate", &self.bit_rate())
            .field("streams", &self.streams())
            .finish_non_exhaustive()
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
}

#[repr(C)]
pub struct Stream {
    raw: NonNull<AVStream>,
}

implement_raw!(Stream: AVStream);

unsafe impl Send for Stream {}
unsafe impl Sync for Stream {}

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
pub struct VideoPixelDescriptor(AVPixFmtDescriptor);

unsafe impl Sync for VideoPixelDescriptor {}

impl VideoPixelDescriptor {
    /// Format name string
    pub const fn name(&self) -> &'static str {
        let name_cstr = unsafe { CStr::from_ptr(self.0.name) };
        unsafe { str::from_utf8_unchecked(name_cstr.to_bytes()) }
    }

    /// Amount to shift the luma width right to find the chroma width.
    /// For YV12 this is 1 for example.
    /// This value only refers to the chroma components.
    pub const fn log2_chroma_width(&self) -> u8 {
        self.0.log2_chroma_w
    }

    /// Amount to shift the luma height right to find the chroma height.
    /// For YV12 this is 1 for example.
    /// This value only refers to the chroma components.
    pub const fn log2_chroma_height(&self) -> u8 {
        self.0.log2_chroma_h
    }

    /// Pixel format flags
    pub const fn flags(&self) -> PixelFormatFlags {
        let bits = self.0.flags;
        unsafe { PixelFormatFlags::from_bits(bits).unwrap_unchecked() }
    }

    /// Alternative comma-separated names.
    pub const fn alias(&self) -> Option<&'static str> {
        let ptr = self.0.alias;

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
        let ptr = &raw const self.0.comp[0];
        let count = self.0.nb_components;
        unsafe { slice::from_raw_parts(ptr, count as usize) }
    }
}

impl fmt::Debug for VideoPixelDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VideoPixelDescriptor")
            .field("name", &self.name())
            .field("log2_chroma_w", &self.log2_chroma_width())
            .field("log2_chroma_h", &self.log2_chroma_height())
            .field("flags", &self.flags())
            .field("alias", &self.alias())
            .field("components", &self.components())
            .finish()
    }
}

/// This structure describes decoded (raw) audio or video data.
///
/// AVFrame must be allocated using av_frame_alloc(). Note that this only
/// allocates the AVFrame itself, the buffers for the data must be managed
/// through other means (see below).
/// AVFrame must be freed with av_frame_free().
///
/// AVFrame is typically allocated once and then reused multiple times to hold
/// different data (e.g. a single AVFrame to hold frames received from a
/// decoder). In such a case, av_frame_unref() will free any references held by
/// the frame and reset it to its original clean state before it
/// is reused again.
///
/// The data described by an AVFrame is usually reference counted through the
/// AVBuffer API. The underlying buffer references are stored in AVFrame.buf /
/// AVFrame.extended_buf. An AVFrame is considered to be reference counted if at
/// least one reference is set, i.e. if AVFrame.buf[0](0) != NULL. In such a case,
/// every single data plane must be contained in one of the buffers in
/// AVFrame.buf or AVFrame.extended_buf.
/// There may be a single buffer for all the data, or one separate buffer for
/// each plane, or anything in between.
///
/// sizeof(AVFrame) is not a part of the public ABI, so new fields may be added
/// to the end with a minor bump.
///
/// Fields can be accessed through AVOptions, the name string used, matches the
/// C structure field name for fields accessible through AVOptions.
pub struct Frame {
    raw: NonNull<AVFrame>,
}

implement_raw!(Frame: AVFrame);

unsafe impl Send for Frame {}
unsafe impl Sync for Frame {}

impl Frame {
    /// Allocate an [`Frame`] and set its fields to default values.
    pub fn new() -> Self {
        let Some(raw) = NonNull::new(unsafe { av_frame_alloc() }) else {
            panic!("av_frame_alloc() failed");
        };

        Self { raw }
    }

    /// Width of the video in pixels
    pub const fn width(&self) -> u32 {
        match unsafe { (*self.as_raw().as_ptr()).width } {
            ..0 => unsafe { hint::unreachable_unchecked() },
            non_negative @ 0.. => non_negative.cast_unsigned(),
        }
    }

    /// Height of the video in pixels
    pub const fn height(&self) -> u32 {
        match unsafe { (*self.as_raw().as_ptr()).height } {
            ..0 => unsafe { hint::unreachable_unchecked() },
            non_negative @ 0.. => non_negative.cast_unsigned(),
        }
    }

    /// Dimensions of the video in pixels
    pub const fn size(&self) -> UVec2 {
        UVec2::new(self.width(), self.height())
    }

    /// Format of each video pixel
    ///
    /// # Note
    ///
    /// Returns [`None`] if unknown or unset.
    pub const fn format(&self) -> Option<VideoPixelFormat> {
        VideoPixelFormat::from_i32(unsafe { (*self.as_raw().as_ptr()).format })
    }

    /// Frame duration in `base` units
    ///
    /// # Note
    ///
    /// Returns [`None`] if unknown
    pub const fn duration_in(&self, base: RatioI32) -> Option<FrameDuration> {
        let Some(duration) = NonZeroI64::new(unsafe { (*self.as_raw().as_ptr()).duration }) else {
            return None;
        };

        Some(FrameDuration { base, duration })
    }

    /// Number of data planes in the [`Frame`]
    pub fn count_planes(&self) -> usize {
        const MAX_N_PLANES: usize = 8;

        for i in 0..MAX_N_PLANES {
            if unsafe { (*self.as_raw().as_ptr()).linesize[i] } == 0 {
                return i;
            }
        }

        MAX_N_PLANES
    }

    /// Width of `index`th data plane in pixels
    ///
    /// # Note
    ///
    /// Returns `0` if `index` is not smaller than number of planes
    pub fn plane_width(&self, index: usize) -> u32 {
        if index >= self.count_planes() {
            return 0;
        }

        // Logic taken from image_get_linesize().
        if index != 1 && index != 2 {
            return self.width();
        }

        let Some(desc) = self.format().unwrap().descriptor() else {
            return self.width();
        };

        let s = desc.log2_chroma_width();
        (self.width() + (1 << s) - 1) >> s
    }

    /// Height of `index`th data plane in pixels
    ///
    /// # Note
    ///
    /// Returns `0` if `index` is not smaller than number of planes
    pub fn plane_height(&self, index: usize) -> u32 {
        if index >= self.count_planes() {
            return 0;
        }

        if index != 1 && index != 2 {
            return self.height();
        }

        let Some(desc) = self.format().unwrap().descriptor() else {
            return self.height();
        };

        let s = desc.log2_chroma_height();
        (self.height() + (1 << s) - 1) >> s
    }

    /// Number of bytes in each row of `index`th plane
    ///
    /// # Panic
    ///
    /// Panics if `index` is not smaller than number of planes
    pub fn stride(&self, index: usize) -> usize {
        if index >= self.count_planes() {
            panic!("out of bounds");
        }

        let line_size = unsafe { (*self.as_raw().as_ptr()).linesize[index] };
        assert!(line_size > 0, "negative stride is unimplemented");

        line_size as usize
    }

    /// Frame data at `index`th plane
    ///
    /// # Note
    ///
    /// Returns `&[]` if `index` is not smaller than number of planes
    pub fn data(&self, index: usize) -> &[u8] {
        if index >= self.count_planes() {
            return &[];
        }

        unsafe {
            slice::from_raw_parts(
                (*self.as_raw().as_ptr()).data[index],
                self.stride(index) * self.plane_height(index) as usize,
            )
        }
    }

    /// Frame is empty completely (no data in all planes)
    pub fn is_empty(&self) -> bool {
        unsafe { (*self.as_raw().as_ptr()).data[0] }.is_null()
    }

    /// # Safety
    ///
    /// [`Frame`] should not be allocated, otherwise memory leak
    pub unsafe fn alloc(
        &mut self,
        format: VideoPixelFormat,
        size: UVec2,
    ) -> Result<(), BackendError> {
        let this = self.as_raw().as_ptr();

        unsafe { (*this).format = format as i32 };
        unsafe { (*this).width = size.x as i32 };
        unsafe { (*this).height = size.y as i32 };

        BackendError::result_of(unsafe { av_frame_get_buffer(self.as_raw().as_ptr(), 0) })
    }

    /// Checks if [`Frame`] is not reference_counted
    pub fn is_owned(&self) -> bool {
        let ptr = unsafe { (*self.as_raw().as_ptr()).buf[0] };

        if ptr.is_null() {
            return true;
        }

        let reference_count = match unsafe { av_buffer_get_ref_count(ptr) } {
            non_negative @ 0.. => non_negative as usize,
            ..0 => unsafe { hint::unreachable_unchecked() },
        };

        reference_count <= 1
    }

    /// # Safety
    ///
    /// The frame is created by hardware-accelerated (by libva) [`CodecContext`]
    pub unsafe fn surface_id(&self) -> va::SurfaceId {
        unsafe { (*self.as_raw().as_ptr()).data[3] as usize as va::SurfaceId }
    }
}

impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Frame")
            .field("size", &self.size())
            .field("format", &self.format())
            .field("plane_count", &self.count_planes())
            .finish_non_exhaustive()
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        let mut ptr = self.raw.as_ptr();

        if self.is_owned() {
            unsafe { av_frame_free(&raw mut ptr) };
        } else {
            unsafe { av_frame_unref(ptr) };
        }
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

/// This structure stores compressed data. It is typically exported by demuxers
/// and then passed as input to decoders, or received as output from encoders and
/// then passed to muxers.
///
/// For video, it should typically contain one compressed frame. For audio it may
/// contain several compressed frames. Encoders are allowed to output empty
/// packets, with no compressed data, containing only side data
/// (e.g. to update some stream parameters at the end of encoding).
///
/// The semantics of data ownership depends on the buf field.
/// If it is set, the packet data is dynamically allocated and is
/// valid indefinitely until a call to av_packet_unref() reduces the
/// reference count to 0.
///
/// If the buf field is not set av_packet_ref() would make a copy instead
/// of increasing the reference count.
///
/// The side data is always allocated with av_malloc(), copied by
/// av_packet_ref() and freed by av_packet_unref().
pub struct Packet {
    raw: NonNull<AVPacket>,
}

implement_raw!(Packet: AVPacket);

unsafe impl Send for Packet {}
unsafe impl Sync for Packet {}

impl Packet {
    /// Creates new empty [`Packet`]
    pub fn new() -> Self {
        let Some(raw) = NonNull::new(unsafe { av_packet_alloc() }) else {
            panic!("failed to allocate new packet");
        };

        Self { raw }
    }

    /// Initializes packet with default values
    pub fn init(&mut self, size: usize) -> Result<(), BackendError> {
        BackendError::result_of(unsafe {
            av_new_packet(self.as_raw().as_ptr(), size.try_into().unwrap())
        })
    }

    /// Creates new default-initialized [`Packet`]
    pub fn with_capacity(size: usize) -> Result<Self, BackendError> {
        let mut result = Self::new();
        result.init(size)?;
        Ok(result)
    }

    /// Setup a new reference to the data described by a given packet
    ///
    /// If src is reference-counted, setup dst as a new reference to the
    /// buffer in src. Otherwise allocate a new buffer in dst and copy the
    /// data from src into it.
    ///
    /// All the other fields are copied from src.
    pub fn try_ref(&mut self) -> Result<Self, BackendError> {
        let packet_ptr = Self::new();
        BackendError::result_of(unsafe {
            av_packet_ref(
                packet_ptr.as_raw().as_ptr(),
                self.as_raw().as_ptr().cast_const(),
            )
        })
        .map(|()| packet_ptr)
    }

    /// Index of the stream this packet belongs to
    pub const fn stream_index(&self) -> usize {
        match unsafe { (*self.as_raw().as_ptr()).stream_index } {
            ..0 => unsafe { hint::unreachable_unchecked() },
            non_negative @ 0.. => non_negative as usize,
        }
    }

    /// Checks if [`Packet`] is not reference-counted
    pub fn is_owned(&self) -> bool {
        let ptr = unsafe { (*self.as_raw().as_ptr()).buf };

        if ptr.is_null() {
            return true;
        }

        let reference_count = match unsafe { av_buffer_get_ref_count(ptr) } {
            non_negative @ 0.. => non_negative as usize,
            ..0 => unsafe { hint::unreachable_unchecked() },
        };

        reference_count == 1
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Packet")
            .field("stream_index", &self.stream_index())
            .finish_non_exhaustive()
    }
}

impl Drop for Packet {
    fn drop(&mut self) {
        let mut ptr = self.as_raw().as_ptr();

        // NOTE(hack3rmann): if the packet is reference-counted then just unref it
        if self.is_owned() {
            unsafe { av_packet_free(&raw mut ptr) };
        } else {
            unsafe { av_packet_unref(ptr) };
        }
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

unsafe impl Send for Profile {}
unsafe impl Sync for Profile {}

pub static PROFILE_UNKNOWN: Profile = Profile::UNKNOWN;

impl Profile {
    pub const UNKNOWN: Self = Self(AVProfile {
        profile: AV_PROFILE_UNKNOWN,
        name: ptr::null(),
    });

    /// Profile ID
    pub const fn id(self) -> i32 {
        self.0.profile
    }

    /// Short name for the profile
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

/// Iterator yielding [`Profile`]s
pub struct ProfileIterator<'s> {
    ptr: NonNull<AVProfile>,
    _p: PhantomData<&'s Codec>,
}

// Safety: `Codec` is `Sync` therefore `ProfileIterator` is `Send` and `Sync`
unsafe impl Send for ProfileIterator<'_> {}
unsafe impl Sync for ProfileIterator<'_> {}

impl<'s> ProfileIterator<'s> {
    /// # Safety
    ///
    /// - `ptr` should point to a valid [`AVProfile`] value
    /// - lifetime of `Self` should be appropriate
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

/// Iterator yielding all supported [`Codec`]s on this device
pub struct CodecIterator {
    raw: *mut c_void,
}

unsafe impl Send for CodecIterator {}
unsafe impl Sync for CodecIterator {}

impl CodecIterator {
    /// Constructs new [`CodecIterator`]
    pub const fn new() -> Self {
        CodecIterator {
            raw: ptr::null_mut(),
        }
    }

    /// Handy filter for decoders
    pub fn decoders() -> impl Iterator<Item = &'static Codec> {
        Self::new().filter(|c| c.is_decoder())
    }

    /// Handy filter for encoders
    pub fn encoders() -> impl Iterator<Item = &'static Codec> {
        Self::new().filter(|c| c.is_encoder())
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
        unsafe { codec.cast::<Codec>().as_ref() }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

pub struct SoftwareScaler {
    raw: NonNull<SwsContext>,
    source_format: ScalerFormat,
    destination_format: ScalerFormat,
}

unsafe impl Send for SoftwareScaler {}
unsafe impl Sync for SoftwareScaler {}

impl SoftwareScaler {
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

        if output.is_empty() {
            // Safety: frame is not allocated
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
