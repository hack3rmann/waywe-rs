//! This crate contains useful functions that extend functionality of the safe_transmute crate

#[derive(thiserror::Error, Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Default)]
#[error("failed to transmute Vec<u8> to Vec<u32>, input data is likely corrupt")]
pub struct TransmuteVecError;

/// This function transmutes the vec of bytes to the vec of u32
///
/// # Return value
///
/// - Returns error if either length or capacity of the vector are not a multiple of 4
/// - Reinterprets bytes of the vector if it is properly aligned for u32 and manually constructs u32s otherwise
pub fn transmute_vec(mut src: Vec<u8>) -> Result<Vec<u32>, TransmuteVecError> {
    if src.len() % 4 != 0 || src.capacity() % 4 != 0 {
        return Err(TransmuteVecError);
    }

    let ptr = src.as_ptr();

    if !(ptr.cast::<u32>().is_aligned()) {
        Ok(src
            .chunks(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().expect("chunks have the size of 4")))
            .collect())
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
            Ok(Vec::from_raw_parts(ptr.cast(), len / 4, capacity / 4))
        }
    }
}

// TODO(ArnoDarkrose): write documentation and safety for unsafe block
pub fn transmute_to_bytes_vec(mut src: Vec<u32>) -> Result<Vec<u8>, TransmuteVecError> {
    let ptr = src.as_ptr();

    if !ptr.cast::<u8>().is_aligned() {
        Ok(src.into_iter().flat_map(|num| num.to_le_bytes()).collect())
    } else {
        unsafe {
            let capacity = src.capacity();
            let len = src.len();
            let ptr = src.as_mut_ptr();
            std::mem::forget(src);
            Ok(Vec::from_raw_parts(ptr.cast(), len * 4, capacity * 4))
        }
    }
}

/// Reads exactly `n` bytes from `src` into an initialized buffer
/// and returns it as a [`Vec`]
pub fn read_into_uninit(src: &mut impl std::io::Read, n: usize) -> std::io::Result<Vec<u8>> {
    let mut data = Box::new_uninit_slice(n);
    let ptr = data.as_mut_ptr().cast();

    // Safety
    //
    // - `ptr` is non-null and aligned as it was allocated via a call to `Box::new_uninit_slice`
    // - `ptr` is allocated within a single object via a call to `Box::new_uinit_slice`
    // - `ptr` points to n elements as this exact value was passed to `new_uninit_slice`
    // - there are no other pointers that access this memory while this slice is being operated on
    // - `Box::new_uninit_slice` wouldn't let to create an allocation that is more than isize::MAX bytes
    let dest_slice = unsafe { std::slice::from_raw_parts_mut(ptr, n) };

    src.read_exact(dest_slice)?;

    // Safety
    //
    // The contents of the slice were initialized above
    let data = unsafe { data.assume_init() };

    Ok(Vec::from(data))
}
