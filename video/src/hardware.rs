use crate::{BackendError, Codec, VideoPixelFormat, implement_raw};
use ffmpeg_sys_next::{
    AVBufferRef, AVCodecContext, AVCodecHWConfig, AVHWDeviceType, AVPixelFormat, av_buffer_unref,
    av_hwdevice_ctx_create, av_hwdevice_get_type_name, av_hwdevice_iterate_types,
};
use std::{
    ffi::CStr,
    fmt,
    ptr::{self, NonNull},
};

/// Type of hardware acceleration provider
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Default)]
pub enum HardwareDeviceType {
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

impl HardwareDeviceType {
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

impl From<HardwareDeviceType> for AVHWDeviceType {
    fn from(value: HardwareDeviceType) -> Self {
        value.to_backend()
    }
}

/// Iterator yieding supported [`HardwareDeviceType`]s
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HwDeviceTypeIterator {
    current: AVHWDeviceType,
}
static_assertions::assert_impl_all!(HwDeviceTypeIterator: Send, Sync);

impl HwDeviceTypeIterator {
    pub const fn new() -> Self {
        Self {
            current: AVHWDeviceType::AV_HWDEVICE_TYPE_NONE,
        }
    }
}

impl Default for HwDeviceTypeIterator {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for HwDeviceTypeIterator {
    type Item = HardwareDeviceType;

    fn next(&mut self) -> Option<Self::Item> {
        self.current = unsafe { av_hwdevice_iterate_types(self.current) };
        HardwareDeviceType::from_backend(self.current)
    }
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
    pub struct HardwareConfigMethods: i32 {
        const HW_DEVICE_CONTEXT = 1;
        const HW_FRAMES_CONTEXT = 2;
        const INTERNAL = 4;
        const AD_HOC = 8;
    }
}

impl HardwareConfigMethods {
    /// Contains either [`HardwareConfigMethods::HW_DEVICE_CONTEXT`]
    /// or [`HardwareConfigMethods::HW_FRAMES_CONTEXT`]
    pub const fn is_hardware(self) -> bool {
        (self.bits() & Self::HW_DEVICE_CONTEXT.bits()) != 0
            || (self.bits() & Self::HW_FRAMES_CONTEXT.bits()) != 0
    }

    /// Contains [`HardwareConfigMethods::INTERNAL`]
    pub const fn is_internal(self) -> bool {
        (self.bits() & Self::INTERNAL.bits()) != 0
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct HardwareConfig(AVCodecHWConfig);
static_assertions::assert_impl_all!(HardwareConfig: Send, Sync);

impl HardwareConfig {
    /// A hardware pixel format which that decoder may be
    /// able to decode to if suitable hardware is available.
    pub const fn format(&self) -> Option<VideoPixelFormat> {
        VideoPixelFormat::from_backend(self.0.pix_fmt)
    }

    /// Bit set, describing the possible setup methods
    /// which can be used with this configuration.
    pub const fn methods(&self) -> HardwareConfigMethods {
        unsafe { HardwareConfigMethods::from_bits(self.0.methods).unwrap_unchecked() }
    }

    /// The device type associated with the configuration.
    ///
    /// # Note
    ///
    /// Returns [`None`] if unused.
    pub const fn device_type(&self) -> Option<HardwareDeviceType> {
        if self.methods().is_hardware() {
            Some(unsafe { HardwareDeviceType::from_backend(self.0.device_type).unwrap_unchecked() })
        } else {
            None
        }
    }
}

impl fmt::Debug for HardwareConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HardwareConfig")
            .field("format", &self.format())
            .field("methods", &self.methods())
            .field("device_type", &self.device_type())
            .finish()
    }
}

/// Iterator yieding [`HardwareConfig`] for the given [`Codec`]
pub struct HardwareConfigIterator<'s> {
    index: usize,
    codec: &'s Codec,
}
static_assertions::assert_impl_all!(HardwareConfigIterator<'_>: Send, Sync);

impl<'s> HardwareConfigIterator<'s> {
    /// Construct new [`HardwareConfigIterator`] for the `codec`
    pub const fn new(codec: &'s Codec) -> Self {
        Self {
            index: usize::MAX,
            codec,
        }
    }
}

impl<'s> Iterator for HardwareConfigIterator<'s> {
    type Item = &'s HardwareConfig;

    fn next(&mut self) -> Option<Self::Item> {
        self.index = self.index.wrapping_add(1);
        self.codec.hardware_config_for_index(self.index)
    }
}

/// Context for the hardware acceleration support
pub struct HardwareDeviceContext {
    raw: NonNull<AVBufferRef>,
}

unsafe impl Send for HardwareDeviceContext {}
unsafe impl Sync for HardwareDeviceContext {}

implement_raw!(HardwareDeviceContext: AVBufferRef);

impl HardwareDeviceContext {
    /// # Safety
    ///
    /// - should be called after [`avcodec_alloc_context3`](ffmpeg_sys_next::avcodec_alloc_context3)
    /// - should be called before [`avcodec_parameters_to_context`](ffmpeg_sys_next::avcodec_parameters_to_context)
    pub unsafe fn new_on_codec(codec_context: NonNull<AVCodecContext>) -> Result<(), BackendError> {
        let mut device_ctx_ptr = ptr::null_mut();

        BackendError::result_of(unsafe {
            av_hwdevice_ctx_create(
                &raw mut device_ctx_ptr,
                HardwareDeviceType::VaApi.to_backend(),
                ptr::null(),
                ptr::null_mut(),
                0,
            )
        })?;

        // NOTE(hack3rmann): libav takes ownership of `Self` therefore we do not return `Self`
        unsafe { (*codec_context.as_ptr()).hw_device_ctx = device_ctx_ptr };
        unsafe { (*codec_context.as_ptr()).get_format = Some(Self::get_hw_format) };

        Ok(())
    }

    pub(crate) unsafe extern "C" fn get_hw_format(
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
}

impl Drop for HardwareDeviceContext {
    fn drop(&mut self) {
        let mut ptr = self.raw.as_ptr();
        unsafe { av_buffer_unref(&raw mut ptr) };
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        BackendError, Codec, CodecContext, CodecIterator, FormatContext, Frame, MediaType, Packet,
    };

    #[test]
    #[ignore = "no file provided"]
    fn hwdec() {
        let mut format_context = FormatContext::from_input(
            c"/home/hack3rmann/Downloads/alone-hollow-knight.3840x2160.mp4",
        )
        .unwrap();

        let best_stream = format_context.find_best_stream(MediaType::Video).unwrap();

        let best_stream_index = best_stream.index();
        let codec_parameters = best_stream.codec_parameters();

        let decoder = Codec::find_decoder_for_id(codec_parameters.codec_id()).unwrap();

        let mut codec_context =
            CodecContext::from_parameters_with_hw_accel(codec_parameters, Some(decoder)).unwrap();

        for config in decoder.hardware_config() {
            dbg!(config);
        }

        codec_context.open(decoder).unwrap();

        let mut maybe_packet = None::<Packet>;
        let mut frame = Frame::new();

        loop {
            if maybe_packet.is_none() {
                let packet = loop {
                    let packet = match format_context.read_packet() {
                        Ok(packet) => packet,
                        Err(BackendError::EOF) => {
                            format_context.repeat_stream(best_stream_index).unwrap();
                            continue;
                        }
                        result @ Err(..) => result.unwrap(),
                    };

                    if packet.stream_index() == best_stream_index {
                        break packet;
                    }
                };

                codec_context.send_packet(&packet).unwrap();

                _ = maybe_packet.insert(packet);
            }

            match codec_context.receive_frame(&mut frame) {
                Ok(()) => break,
                Err(..) => {
                    maybe_packet = None;
                    continue;
                }
            }
        }
    }

    #[test]
    fn print_codec() {
        for codec in CodecIterator::decoders().filter(|c| c.name().contains("264")) {
            dbg!(codec);
        }
    }
}
