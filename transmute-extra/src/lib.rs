//! This crate contains useful functions that extend functionality of the safe_transmute crate

/// This function transmutes the vec of bytes to the vec of u32
///
/// # Return value
///
/// - Returns error if either length or capacity of the vector are not a multiple of 4
/// - Reinterprets bytes of the vector if it is properly aligned for u32 and manually constructs u32s otherwise
pub fn transmute_vec(mut src: Vec<u8>) -> Option<Vec<u32>> {
    if src.len() % 4 != 0 || src.capacity() % 4 != 0 {
        return None;
    }

    let ptr = src.as_ptr();

    if !(ptr.cast::<u32>().is_aligned()) {
        Some(
            src.chunks(4)
                .map(|chunk| {
                    u32::from_le_bytes(chunk.try_into().expect("chunks have the size of 4"))
                })
                .collect(),
        )
    } else {
        // Safety
        //
        // - the pointer was previously allocated via another vec
        // - the alignemnt is checked above
        // - capacity value is adjusted from the valid one (u32 is 4 times bigger than u8)
        // - len is less than or equal to capacity (this is ensured by the original vector)
        // - elements can be transmuted from u8 to u32Z
        // - capacity is not changed
        // - allocated size in bytes cannot exceed isize::MAX. This is ensured by the original vector
        unsafe {
            let capacity = src.capacity();
            let len = src.len();
            let ptr = src.as_mut_ptr();
            std::mem::forget(src);
            Some(Vec::from_raw_parts(ptr.cast(), len / 4, capacity / 4))
        }
    }
}
