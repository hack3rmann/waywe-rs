use std::mem::{self, MaybeUninit};

pub trait BoxExt {
    type Inner;

    fn into_inner(self) -> Self::Inner;
}

impl<T> BoxExt for Box<T> {
    type Inner = T;

    fn into_inner(self) -> Self::Inner
    where
        Self: Sized,
    {
        // Safety:
        // - `Self` has valid `MaybeUninit<Self>` bytes
        // - `Box<MaybeUninit<..>>` will deallocate memory without dropping the value of `Self`
        let boxed_uninit = unsafe { mem::transmute::<Box<T>, Box<MaybeUninit<T>>>(self) };

        // Safety: the value was valid
        unsafe { boxed_uninit.assume_init_read() }
    }
}
