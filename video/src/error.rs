use ffmpeg_sys_next::{
    AVERROR_BSF_NOT_FOUND, AVERROR_BUFFER_TOO_SMALL, AVERROR_BUG, AVERROR_BUG2,
    AVERROR_DECODER_NOT_FOUND, AVERROR_DEMUXER_NOT_FOUND, AVERROR_ENCODER_NOT_FOUND, AVERROR_EOF,
    AVERROR_EXIT, AVERROR_EXTERNAL, AVERROR_FILTER_NOT_FOUND, AVERROR_HTTP_BAD_REQUEST,
    AVERROR_HTTP_FORBIDDEN, AVERROR_HTTP_NOT_FOUND, AVERROR_HTTP_OTHER_4XX,
    AVERROR_HTTP_SERVER_ERROR, AVERROR_HTTP_TOO_MANY_REQUESTS, AVERROR_HTTP_UNAUTHORIZED,
    AVERROR_INVALIDDATA, AVERROR_MUXER_NOT_FOUND, AVERROR_OPTION_NOT_FOUND, AVERROR_PATCHWELCOME,
    AVERROR_PROTOCOL_NOT_FOUND, AVERROR_STREAM_NOT_FOUND, AVERROR_UNKNOWN, av_strerror, strerror,
};
use std::{error::Error, ffi::CStr, fmt, num::NonZeroI32};

/// Backend error from libav
#[derive(Clone, Copy, PartialEq)]
pub struct BackendError {
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

    /// Constructs [`Result`] from libav status code
    pub const fn result_of(code: i32) -> Result<(), Self> {
        match Self::new(code) {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    /// Returns error if `code` is negative or a non-negative number instead
    pub const fn result_or_u32(code: i32) -> Result<u32, Self> {
        match Self::new(code) {
            Some(error) => Err(error),
            None => Ok(code.cast_unsigned()),
        }
    }

    /// Returns error if `code` is negative or a non-negative number instead
    ///
    /// # Panic
    ///
    /// Panics if `code < i32::MIN`
    pub const fn result_or_u64(code: i64) -> Result<u64, Self> {
        const I32_MIN: i64 = i32::MIN as i64;

        match code {
            error @ I32_MIN..0 => Err(unsafe { Self::new(error as i32).unwrap_unchecked() }),
            non_negative @ 0.. => Ok(non_negative.cast_unsigned()),
            ..I32_MIN => panic!("error code out of range"),
        }
    }

    /// Constructs new [`BackendError`] from libav status code
    pub const fn new(code: i32) -> Option<Self> {
        match code {
            ..0 => Some(Self {
                encoded_code: unsafe { NonZeroI32::new_unchecked(code) },
            }),
            0.. => None,
        }
    }

    /// Error code
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

    /// String with the error description
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
