use crate::{
    AudioVideoFormat, BackendError, Frame, HardwareDeviceType, MediaType, Packet, Profile,
    ProfileIterator, RatioI32, Stream,
    hardware::{HardwareConfig, HardwareConfigIterator},
    implement_raw,
};
use bitflags::bitflags;
use ffmpeg_sys_next::{
    AV_PROFILE_UNKNOWN, AVCodec, AVCodecContext, AVCodecID, AVCodecParameters, AVPixelFormat,
    av_buffer_ref, av_codec_is_decoder, av_codec_is_encoder, av_hwdevice_ctx_create,
    avcodec_alloc_context3, avcodec_find_decoder, avcodec_get_hw_config, avcodec_open2,
    avcodec_parameters_alloc, avcodec_parameters_copy, avcodec_parameters_free,
    avcodec_parameters_to_context, avcodec_receive_frame, avcodec_send_packet,
};
use glam::UVec2;
use std::{
    borrow::Borrow,
    ffi::CStr,
    fmt,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
    slice,
};

#[repr(transparent)]
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

impl ToOwned for CodecParameters {
    type Owned = OwnedCodecParameters;

    fn to_owned(&self) -> Self::Owned {
        match OwnedCodecParameters::try_from(self) {
            Err(error) => panic!("failed to clone CodecParameters: {error:?}"),
            Ok(result) => result,
        }
    }
}

pub struct OwnedCodecParameters {
    raw: NonNull<AVCodecParameters>,
}

implement_raw!(OwnedCodecParameters: AVCodecParameters);

impl OwnedCodecParameters {
    pub fn new() -> Self {
        let Some(raw) = NonNull::new(unsafe { avcodec_parameters_alloc() }) else {
            panic!("failed to allocate avcodec parameters");
        };

        Self { raw }
    }

    pub fn try_clone_from(&mut self, other: &CodecParameters) -> Result<(), BackendError> {
        BackendError::result_of(unsafe {
            avcodec_parameters_copy(self.as_raw().as_ptr(), (&raw const *other).cast())
        })
    }
}

impl fmt::Debug for OwnedCodecParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.deref(), f)
    }
}

impl Borrow<CodecParameters> for OwnedCodecParameters {
    fn borrow(&self) -> &CodecParameters {
        self
    }
}

impl Clone for OwnedCodecParameters {
    fn clone_from(&mut self, source: &Self) {
        if let Err(error) = self.try_clone_from(source) {
            panic!("failed to clone CodecParameters: {error:?}");
        }
    }

    fn clone(&self) -> Self {
        let mut result = Self::new();
        result.clone_from(self);
        result
    }
}

impl Drop for OwnedCodecParameters {
    fn drop(&mut self) {
        let mut ptr = self.raw.as_ptr();
        unsafe { avcodec_parameters_free(&raw mut ptr) };
    }
}

impl Deref for OwnedCodecParameters {
    type Target = CodecParameters;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.cast::<CodecParameters>().as_ref() }
    }
}

impl DerefMut for OwnedCodecParameters {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.raw.cast::<CodecParameters>().as_mut() }
    }
}

impl TryFrom<&'_ CodecParameters> for OwnedCodecParameters {
    type Error = BackendError;

    fn try_from(value: &'_ CodecParameters) -> Result<Self, Self::Error> {
        let mut result = Self::new();
        result.try_clone_from(value).map(|()| result)
    }
}

impl Default for OwnedCodecParameters {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CodecContext {
    raw: NonNull<AVCodecContext>,
}

unsafe impl Send for CodecContext {}

implement_raw!(CodecContext: AVCodecContext);

impl CodecContext {
    pub fn from_parameters(parameters: &CodecParameters) -> Result<Self, BackendError> {
        // FIXME(hack3rmann): may result in suboptimal behavior ((C) docs)
        let Some(codec_context_ptr) = NonNull::new(unsafe { avcodec_alloc_context3(ptr::null()) })
        else {
            panic!("unexpected libav error");
        };

        let mut device_ctx_ptr = ptr::null_mut();

        // FIXME(hack3rmann): memory leak here
        BackendError::result_of(unsafe {
            av_hwdevice_ctx_create(
                &raw mut device_ctx_ptr,
                HardwareDeviceType::VaApi.to_backend(),
                ptr::null(),
                ptr::null_mut(),
                0,
            )
        })?;

        unsafe extern "C" fn get_hw_format(
            _: *mut AVCodecContext,
            formats: *const AVPixelFormat,
        ) -> AVPixelFormat {
            let mut format_ptr = formats;

            loop {
                let format = unsafe { format_ptr.read() };

                if let AVPixelFormat::AV_PIX_FMT_VAAPI | AVPixelFormat::AV_PIX_FMT_NONE = format {
                    break format;
                }

                format_ptr = format_ptr.wrapping_add(1);
            }
        }

        unsafe { (*codec_context_ptr.as_ptr()).hw_device_ctx = av_buffer_ref(device_ctx_ptr) };
        unsafe { (*codec_context_ptr.as_ptr()).get_format = Some(get_hw_format) };

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

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct CodecId(pub AVCodecID);

impl fmt::Debug for CodecId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Default, Ord, Hash)]
    pub struct CodecCapability: i32 {
        const DRAW_HORIZ_BAND = 1;
        const DR1 = 2;
        const DELAY = 32;
        const SMALL_LAST_FRAME = 64;
        const SUBFRAMES = 256;
        const EXPERIMENTAL = 512;
        const CHANNEL_CONF = 1024;
        const FRAME_THREADS = 4096;
        const SLICE_THREADS = 8192;
        const PARAM_CHANGE = 16384;
        const OTHER_THREADS = 32768;
        const VARIABLE_FRAME_SIZE = 65536;
        const AVOID_PROBING = 131072;
        const HARDWARE = 262144;
        const HYBRID = 524288;
        const ENCODER_REORDERED_OPAQUE = 1048576;
        const ENCODER_FLUSH = 2097152;
        const ENCODER_RECON_FRAME = 4194304;
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

    pub const fn capabilities(&self) -> CodecCapability {
        unsafe { CodecCapability::from_bits(self.0.capabilities).unwrap_unchecked() }
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

    pub fn hardware_config_for_index(&self, index: usize) -> Option<&HardwareConfig> {
        let ptr = unsafe { avcodec_get_hw_config((&raw const *self).cast(), index as i32) };
        unsafe { ptr.cast::<HardwareConfig>().as_ref() }
    }

    pub fn hardware_config(&self) -> HardwareConfigIterator<'_> {
        HardwareConfigIterator::new(self)
    }
}

impl fmt::Debug for Codec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Codec")
            .field("name", &self.name())
            .field("long_name", &self.long_name())
            .field("media_type", &self.media_type())
            .field("id", &self.id())
            .field("capabilities", &self.capabilities())
            .field("wrapper_name", &self.wrapper_name())
            .finish_non_exhaustive()
    }
}
