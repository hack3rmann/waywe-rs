pub mod codec;
pub mod error;
pub mod format;
pub mod hardware_acceleration;
pub mod time;

use bitflags::bitflags;
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

pub use codec::{Codec, CodecContext, CodecId, CodecParameters, OwnedCodecParameters};
pub use error::BackendError;
pub use ffmpeg_sys_next::AVComponentDescriptor as ComponentDescriptor;
pub use format::{AudioVideoFormat, PixelFormatFlags, VideoPixelFormat};
pub use hardware_acceleration::{HwDeviceType, HwDeviceTypeIterator};
pub use time::{FrameDuration, RatioI32};

/// Initialize `libavdevice` and register all the input and output devices.
pub fn init() {
    unsafe { avdevice_register_all() };
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

impl VideoPixelDescriptor {
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

        let s = desc.log2_chroma_height();
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

    pub fn is_owned(&self) -> bool {
        let ptr = unsafe { (*self.as_raw().as_ptr()).buf[0] };

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

pub struct Packet {
    raw: NonNull<AVPacket>,
}

implement_raw!(Packet: AVPacket);

impl Packet {
    pub fn new() -> Self {
        let Some(raw) = NonNull::new(unsafe { av_packet_alloc() }) else {
            panic!("failed to allocate new packet");
        };

        Self { raw }
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

impl Clone for Packet {
    fn clone(&self) -> Self {
        match self.try_clone() {
            Err(error) => panic!("failed to clone a packet: {error:?}"),
            Ok(packet) => packet,
        }
    }
}

impl Drop for Packet {
    fn drop(&mut self) {
        let mut ptr = self.as_raw().as_ptr();

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

impl Profile {
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
