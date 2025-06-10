pub mod ffi;

use std::{ffi::CStr, fmt, str};
use thiserror::Error;

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
