//! This crate contains useful functions that extend functionality of the safe_transmute crate

#[cfg(unix)]
use std::{ffi::CString, path::PathBuf};

#[derive(thiserror::Error, Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Default)]
#[error("failed to transmute Vec<u8> to Vec<u32>, input data is likely corrupt")]
pub struct TransmuteVecError;

/// This function transmutes the vec of bytes to the vec of u32
///
/// # Return value
///
/// - Returns error if either length or capacity of the vector are not a multiple of 4
/// - Reinterprets bytes of the vector if it is properly aligned for u32 and manually constructs u32s otherwise
pub fn transmute_vec_u8_to_vec_u32(mut src: Vec<u8>) -> Result<Vec<u32>, TransmuteVecError> {
    if src.len() % 4 != 0 || src.capacity() % 4 != 0 {
        return Err(TransmuteVecError);
    }

    let ptr = src.as_ptr();

    if !ptr.cast::<u32>().is_aligned() {
        Ok(src
            .chunks(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().expect("chunks have the size of 4")))
            .collect())
    } else {
        let capacity = src.capacity();
        let len = src.len();
        let ptr = src.as_mut_ptr();

        std::mem::forget(src);

        // Safety
        //
        // - the pointer was previously allocated via another vec
        // - the alignemnt is checked above
        // - capacity value is adjusted from the valid one (u32 is 4 times bigger than u8)
        // - len is less than or equal to capacity (this is ensured by the original vector)
        // - elements can be transmuted from u8 to u32
        // - capacity is not changed
        // - allocated size in bytes cannot exceed isize::MAX. This is ensured by the original vector
        Ok(unsafe { Vec::from_raw_parts(ptr.cast(), len / 4, capacity / 4) })
    }
}

/// This function transmutes the vec of u32  to the vec of bytes
///
/// # Return value
///
/// - Reinterprets bytes of the vector if it is properly aligned for u8 and manually constructs u8s otherwise
pub fn transmute_vec_u32_to_vec_u8(mut src: Vec<u32>) -> Vec<u8> {
    let capacity = src.capacity();
    let len = src.len();
    let ptr = src.as_mut_ptr();

    std::mem::forget(src);

    // Safety
    //
    // - the pointer was previously allocated via another vec
    // - the alignemnt is checked above
    // - capacity value is adjusted from the valid one (u32 is 4 times bigger than u8)
    // - len is less than or equal to capacity (this is ensured by the original vector)
    // - elements can be transmuted from u32 to u8
    // - capacity is not changed
    // - allocated size in bytes cannot exceed isize::MAX. This is ensured by the original vector
    unsafe { Vec::from_raw_parts(ptr.cast(), len * 4, capacity * 4) }
}

/// Converts [`PathBuf`] into [`CString`], adding `nul` character at the end.
#[cfg(unix)]
pub fn pathbuf_into_cstring(path: PathBuf) -> CString {
    let mut path = path.into_os_string().into_encoded_bytes();
    path.push(0);

    // Safety:
    // - has `nul` at the end
    // - there is exactly one `nul` character, because it was valid unix-string
    unsafe { CString::from_vec_with_nul_unchecked(path) }
}
