use std::mem::{self, MaybeUninit};

pub trait BoxExt {
    fn into_inner(this: Box<Self>) -> Self
    where
        Self: Sized,
    {
        // Safety:
        // - `Self` has valid `MaybeUninit<Self>` bytes
        // - `Box<MaybeUninit<..>>` will deallocate memory without dropping the value of `Self`
        let boxed_uninit = unsafe { mem::transmute::<Box<Self>, Box<MaybeUninit<Self>>>(this) };

        // Safety: the value was valid
        unsafe { boxed_uninit.assume_init_read() }
    }
}

impl<T> BoxExt for T {}
