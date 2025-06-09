use std::{
    ffi::{CStr, c_char, c_void},
    fmt,
    os::fd::RawFd,
};
use thiserror::Error;

#[derive(Clone, Copy, Error, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum VaError {
    Unknown = -1,
    OperationFailed = 1,
    AllocationFailed = 2,
    InvalidDisplay = 3,
    InvalidConfig = 4,
    InvalidContext = 5,
    InvalidSurface = 6,
    InvalidBuffer = 7,
    InvalidImage = 8,
    InvalidSubpicture = 9,
    AttributeNotSupported = 10,
    MaxumumNumberExceeded = 11,
    UnsupportedProfile = 12,
    UnsupportedEntryPoint = 13,
    UnsupportedRtFormat = 14,
    UnsupportedBufferType = 15,
    SurfaceBusy = 16,
    FlagNotSupported = 17,
    InvalidParameter = 18,
    ResolutionNotSupported = 19,
    Unimplemented = 20,
    SurfaceInDisplaying = 21,
    InvalidImageFormat = 22,
    Decoding = 23,
    Encoding = 24,
    InvalidValue = 25,
    UnsupportedFilter = 32,
    InvalidFilterChain = 33,
    HardwareBusy = 34,
    UnsupportedMemoryType = 36,
    NotEnoughBuffer = 37,
    TimedOut = 38,
}

impl VaError {
    /// Acquire status value from error
    pub const fn status(self) -> VaStatus {
        match self {
            Self::Unknown => -1,
            other => other as VaStatus,
        }
    }

    /// Constructs [`VaError`] from libva [`VaStatus`]
    ///
    /// # Note
    ///
    /// Returns [`None`] if `status` is invalid
    pub const fn new(status: VaStatus) -> Option<Self> {
        Some(match status {
            -1 => Self::Unknown,
            1 => Self::OperationFailed,
            2 => Self::AllocationFailed,
            3 => Self::InvalidDisplay,
            4 => Self::InvalidConfig,
            5 => Self::InvalidContext,
            6 => Self::InvalidSurface,
            7 => Self::InvalidBuffer,
            8 => Self::InvalidImage,
            9 => Self::InvalidSubpicture,
            10 => Self::AttributeNotSupported,
            11 => Self::MaxumumNumberExceeded,
            12 => Self::UnsupportedProfile,
            13 => Self::UnsupportedEntryPoint,
            14 => Self::UnsupportedRtFormat,
            15 => Self::UnsupportedBufferType,
            16 => Self::SurfaceBusy,
            17 => Self::FlagNotSupported,
            18 => Self::InvalidParameter,
            19 => Self::ResolutionNotSupported,
            20 => Self::Unimplemented,
            21 => Self::SurfaceInDisplaying,
            22 => Self::InvalidImageFormat,
            23 => Self::Decoding,
            24 => Self::Encoding,
            25 => Self::InvalidValue,
            32 => Self::UnsupportedFilter,
            33 => Self::InvalidFilterChain,
            34 => Self::HardwareBusy,
            36 => Self::UnsupportedMemoryType,
            37 => Self::NotEnoughBuffer,
            38 => Self::TimedOut,
            _ => return None,
        })
    }

    /// Convertes [`VaStatus`] into result
    ///
    /// # Panic
    ///
    /// Panics if `status` is not a valid libva status
    pub fn result_of(status: VaStatus) -> Result<(), Self> {
        Err(match status {
            -1 => Self::Unknown,
            0 => return Ok(()),
            1 => Self::OperationFailed,
            2 => Self::AllocationFailed,
            3 => Self::InvalidDisplay,
            4 => Self::InvalidConfig,
            5 => Self::InvalidContext,
            6 => Self::InvalidSurface,
            7 => Self::InvalidBuffer,
            8 => Self::InvalidImage,
            9 => Self::InvalidSubpicture,
            10 => Self::AttributeNotSupported,
            11 => Self::MaxumumNumberExceeded,
            12 => Self::UnsupportedProfile,
            13 => Self::UnsupportedEntryPoint,
            14 => Self::UnsupportedRtFormat,
            15 => Self::UnsupportedBufferType,
            16 => Self::SurfaceBusy,
            17 => Self::FlagNotSupported,
            18 => Self::InvalidParameter,
            19 => Self::ResolutionNotSupported,
            20 => Self::Unimplemented,
            21 => Self::SurfaceInDisplaying,
            22 => Self::InvalidImageFormat,
            23 => Self::Decoding,
            24 => Self::Encoding,
            25 => Self::InvalidValue,
            32 => Self::UnsupportedFilter,
            33 => Self::InvalidFilterChain,
            34 => Self::HardwareBusy,
            36 => Self::UnsupportedMemoryType,
            37 => Self::NotEnoughBuffer,
            38 => Self::TimedOut,
            // HACK(hack3rmann): formatting causes this function no be non-const
            other => panic!("unknown libva error code {other}"),
        })
    }

    /// Error description
    pub fn description(self) -> &'static str {
        let error_cstr = unsafe { CStr::from_ptr(vaErrorStr(self.status())) };
        unsafe { std::str::from_utf8_unchecked(error_cstr.to_bytes()) }
    }
}

impl fmt::Debug for VaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VaError")
            .field("code", &self.status())
            .field("description", &self.description())
            .finish()
    }
}

impl fmt::Display for VaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.description())
    }
}

/// Window system dependent
pub type VaDisplay = *mut c_void;
/// Generic ID type, can be re-typed for specific implementation
pub type VaGenericId = u32;
pub type VaSurfaceId = VaGenericId;
/// Return status type from functions
pub type VaStatus = i32;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct VaDrmPrimeSurfaceDescriptorObject {
    /// DRM PRIME file descriptor for this object.
    pub fd: RawFd,
    /// Total size of this object (may include regions which are not part of the surface).
    pub size: u32,
    /// Format modifier applied to this object.
    pub drm_format_modifier: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct VaDrmPrimeSurfaceDescriptorLayer {
    /// DRM format fourcc of this layer (DRM_FOURCC_*).
    pub drm_format: u32,
    /// Number of planes in this layer.
    pub num_planes: u32,
    /// Index in the objects array of the object containing each plane.
    pub object_index: [u32; 4],
    /// Offset within the object of each plane.
    pub offset: [u32; 4],
    /// Pitch of each plane.
    pub pitch: [u32; 4],
}

/// External buffer descriptor for a DRM PRIME surface with flags
///
/// This structure is an extention for VADRMPRIMESurfaceDescriptor,
/// it has the same behavior as if used with VA_SURFACE_ATTRIB_MEM_TYPE_DRM_PRIME_2.
///
/// The field "flags" is added, see "Surface external buffer descriptor flags".
/// To use this structure, use VA_SURFACE_ATTRIB_MEM_TYPE_DRM_PRIME_3 instead.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct VaDrmPrimeDescriptor {
    /// Pixel format fourcc of the whole surface (VA_FOURCC_*).
    pub fourcc: u32,
    /// Width of the surface in pixels.
    pub width: u32,
    /// Height of the surface in pixels.
    pub height: u32,
    /// Number of distinct DRM objects making up the surface.
    pub num_objects: u32,
    /// Description of each object.
    pub objects: [VaDrmPrimeSurfaceDescriptorObject; 4],
    /// Number of layers making up the surface.
    pub num_layers: u32,
    /// Description of each layer in the surface.
    pub layers: [VaDrmPrimeSurfaceDescriptorLayer; 4],
}


/// VAAPI connection details.
/// 
/// Allocated as AVHWDeviceContext.hwctx
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AvVaApiDeviceContext {
    /// The VADisplay handle, to be filled by the user.
    pub display: VaDisplay,
    /// Driver quirks to apply - this is filled by av_hwdevice_ctx_init(),
    /// with reference to a table of known drivers, unless the
    /// AV_VAAPI_DRIVER_QUIRK_USER_SET bit is already present. The user
    /// may need to refer to this field when performing any later
    /// operations using VAAPI with the same VADisplay.
    pub driver_quirks: u32,
}

unsafe extern "C" {
    pub fn vaExportSurfaceHandle(
        display: VaDisplay,
        surface_id: VaSurfaceId,
        memory_type: u32,
        flags: u32,
        descriptor: *mut c_void,
    ) -> VaStatus;

    pub safe fn vaErrorStr(status: VaStatus) -> *const c_char;

    pub fn vaSyncSurface(display: VaDisplay, render_target: VaSurfaceId) -> VaStatus;
}
