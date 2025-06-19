pub mod ffi;

use crate::{CodecContext, implement_raw};
use ffi::DrmPrimeDescriptor;
use ffmpeg_sys_next::AVHWDeviceContext;
use std::{
    ffi::CStr,
    fmt,
    marker::PhantomData,
    mem::MaybeUninit,
    os::fd::{FromRawFd as _, OwnedFd},
    ptr::NonNull,
    str,
};
use thiserror::Error;

pub use ffi::SurfaceId as VaSurfaceId;

/// Error codes for libva backend
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
    pub const fn status(self) -> ffi::Status {
        match self {
            Self::Unknown => -1,
            other => other as ffi::Status,
        }
    }

    /// Constructs [`VaError`] from libva [`VaStatus`]
    ///
    /// # Note
    ///
    /// Returns [`None`] if `status` is invalid
    pub const fn new(status: ffi::Status) -> Option<Self> {
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
    pub fn result_of(status: ffi::Status) -> Result<(), Self> {
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
        let error_cstr = unsafe { CStr::from_ptr(ffi::error_str(self.status())) };
        unsafe { str::from_utf8_unchecked(error_cstr.to_bytes()) }
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

pub struct VaDisplay<'s> {
    raw: NonNull<()>,
    _p: PhantomData<&'s ()>,
}

implement_raw!(VaDisplay<'_> { raw, _p: PhantomData }: ());

impl<'s> VaDisplay<'s> {
    /// `libva` display associated with [`CodecContext`]
    pub fn from_codec_context(context: &'s CodecContext) -> Option<Self> {
        let hw_device_context_buffer = unsafe { (*context.as_raw().as_ptr()).hw_device_ctx };

        if hw_device_context_buffer.is_null() {
            return None;
        }

        let hw_device_context =
            unsafe { (*hw_device_context_buffer).data }.cast::<AVHWDeviceContext>();

        if hw_device_context.is_null() {
            return None;
        }

        let vaapi_device_context = unsafe {
            (*hw_device_context)
                .hwctx
                .cast::<ffi::AvVaApiDeviceContext>()
        };

        if vaapi_device_context.is_null() {
            return None;
        }

        let raw = NonNull::new(unsafe { (*vaapi_device_context).display })?.cast::<()>();

        Some(Self {
            raw,
            _p: PhantomData,
        })
    }

    /// This function blocks until all pending operations on the render target
    /// have been completed. Upon return it is safe to use the render target for a
    /// different picture.
    pub fn sync_surface(&self, id: VaSurfaceId) -> Result<(), VaError> {
        VaError::result_of(unsafe { ffi::sync_surface(self.as_raw().as_ptr().cast(), id) })
    }

    /// Export a handle to a surface for use with an external API
    ///
    /// The exported handles are owned by the caller, and the caller is
    /// responsible for freeing them when no longer needed (e.g. by closing
    /// DRM PRIME file descriptors).
    ///
    /// This does not perform any synchronisation.  If the contents of the
    /// surface will be read, vaSyncSurface() must be called before doing so.
    /// If the contents of the surface are written, then all operations must
    /// be completed externally before using the surface again by via VA-API
    /// functions.
    pub fn export_surface_handle(&self, id: VaSurfaceId) -> Result<VaSurfaceHandle, VaError> {
        const VA_EXPORT_SURFACE_READ_ONLY: u32 = 1;
        const VA_EXPORT_SURFACE_SEPARATE_LAYERS: u32 = 4;

        let desc = {
            // NOTE(hack3rmann): `desc` should be zero-initialized according to the docs
            let mut desc = MaybeUninit::<ffi::DrmPrimeDescriptor>::zeroed();

            VaError::result_of(unsafe {
                ffi::export_surface_handle(
                    self.as_raw().as_ptr().cast(),
                    id,
                    ffi::DrmPrimeDescriptor::LEGACY_MEMORY_TYPE,
                    VA_EXPORT_SURFACE_READ_ONLY | VA_EXPORT_SURFACE_SEPARATE_LAYERS,
                    desc.as_mut_ptr().cast(),
                )
            })?;

            unsafe { desc.assume_init() }
        };

        let fd = unsafe { OwnedFd::from_raw_fd(desc.objects[0].fd) };

        Ok(VaSurfaceHandle { fd, desc })
    }
}

pub struct VaSurfaceHandle {
    fd: OwnedFd,
    desc: DrmPrimeDescriptor,
}

impl VaSurfaceHandle {
    /// Consumes [`VaSurfaceHandle`] leaving only an [`OwnedFd`]
    pub fn into_fd(self) -> OwnedFd {
        self.fd
    }

    /// DRM PRIME descriptor
    pub const fn desc(&self) -> &DrmPrimeDescriptor {
        &self.desc
    }
}
