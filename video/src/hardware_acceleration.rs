use ffmpeg_sys_next::{AVHWDeviceType, av_hwdevice_get_type_name, av_hwdevice_iterate_types};
use std::ffi::CStr;

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
