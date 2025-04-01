//! Implementation of dynamic dispatch with thin pointers

use std::{
    any, fmt, hash,
    mem::{self, MaybeUninit},
    ptr::NonNull,
};

/// Size and offset written in only one [`usize`]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SizeAndOffset(pub usize);

impl SizeAndOffset {
    pub const N_OFFSET_BITS: usize = 4;
    pub const MAX_OFFSET: usize = (1 << Self::N_OFFSET_BITS) - 1;

    /// Constructs new [`SizeAndOffset`] with given `size` and `offset`
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if
    ///
    /// - `size` is too large (unrepresentable in this struct)
    /// - `offset` is larger than `15`
    pub(crate) const fn new(size: usize, offset: usize) -> Option<Self> {
        if offset > Self::MAX_OFFSET {
            return None;
        }

        let encoded_size = size.wrapping_shl(Self::N_OFFSET_BITS as u32);

        if encoded_size.wrapping_shr(Self::N_OFFSET_BITS as u32) != size {
            return None;
        }

        Some(Self(encoded_size | offset))
    }

    pub(crate) const fn size(self) -> usize {
        self.0.wrapping_shr(Self::N_OFFSET_BITS as u32)
    }

    pub(crate) const fn offset(self) -> usize {
        self.0 & Self::MAX_OFFSET
    }
}

impl fmt::Debug for SizeAndOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(any::type_name::<Self>())
            .field("size", &self.size())
            .field("offset", &self.offset())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ThinDataHeader {
    pub drop: unsafe fn(*mut ()),
    pub size_and_offset: SizeAndOffset,
}

impl ThinDataHeader {
    pub const OFFSET_SHIFT: usize = mem::size_of::<ThinDataHeader>();

    pub const fn of<T: 'static>() -> Self {
        // Require Rust to compute it in compile time
        const {
            let drop = |ptr: *mut ()| unsafe {
                ptr.cast::<T>().drop_in_place();
            };

            Self {
                drop,
                size_and_offset: SizeAndOffset::new(
                    mem::size_of::<T>(),
                    mem::offset_of!(ThinData<T>, inner) - Self::OFFSET_SHIFT,
                )
                .unwrap(),
            }
        }
    }

    pub const fn size(self) -> usize {
        self.size_and_offset.size()
    }

    pub const fn offset(self) -> usize {
        self.size_and_offset.offset() + Self::OFFSET_SHIFT
    }
}

/// Wrapper that can be pointed to by a thin pointer
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct ThinData<T: 'static> {
    pub header: ThinDataHeader,
    pub inner: T,
}

impl<T: 'static> ThinData<T> {
    pub(crate) const fn new(data: T) -> Self {
        Self {
            header: const { ThinDataHeader::of::<T>() },
            inner: data,
        }
    }
}

impl<T: Default + 'static> Default for ThinData<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: PartialEq + 'static> PartialEq for ThinData<T> {
    fn eq(&self, other: &Self) -> bool {
        T::eq(&self.inner, &other.inner)
    }

    #[allow(clippy::partialeq_ne_impl)]
    fn ne(&self, other: &Self) -> bool {
        T::ne(&self.inner, &other.inner)
    }
}

impl<T: Eq + 'static> Eq for ThinData<T> {}

impl<T: PartialOrd + 'static> PartialOrd for ThinData<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        T::partial_cmp(&self.inner, &other.inner)
    }

    fn lt(&self, other: &Self) -> bool {
        T::lt(&self.inner, &other.inner)
    }

    fn le(&self, other: &Self) -> bool {
        T::le(&self.inner, &other.inner)
    }

    fn gt(&self, other: &Self) -> bool {
        T::gt(&self.inner, &other.inner)
    }

    fn ge(&self, other: &Self) -> bool {
        T::ge(&self.inner, &other.inner)
    }
}

impl<T: Ord + 'static> Ord for ThinData<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        T::cmp(&self.inner, &other.inner)
    }

    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        Self {
            header: self.header,
            inner: T::max(self.inner, other.inner),
        }
    }

    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        Self {
            header: self.header,
            inner: T::min(self.inner, other.inner),
        }
    }

    fn clamp(self, min: Self, max: Self) -> Self
    where
        Self: Sized,
    {
        Self {
            header: self.header,
            inner: T::clamp(self.inner, min.inner, max.inner),
        }
    }
}

impl<T: hash::Hash + 'static> hash::Hash for ThinData<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        T::hash(&self.inner, state)
    }
}

/// Implementation detail of thin pointers
#[repr(C)]
pub(crate) struct DynThinData {
    pub(crate) header: ThinDataHeader,
    pub(crate) bytes: [MaybeUninit<u8>],
}

impl DynThinData {
    pub unsafe fn from_ptr(ptr: NonNull<()>) -> NonNull<Self> {
        let header = unsafe { ptr.cast::<ThinDataHeader>().as_ref() };

        let n_align_bytese = header.offset() - mem::size_of::<ThinDataHeader>();
        let byte_slice_size = header.size() + n_align_bytese;

        // HACK(hack3rmann): constructing fat pointer like this is non-const
        unsafe { mem::transmute::<(NonNull<()>, usize), NonNull<Self>>((ptr, byte_slice_size)) }
    }

    #[cfg(test)]
    pub fn from_ref<T: 'static>(value: &ThinData<T>) -> &Self {
        unsafe { Self::from_ptr(NonNull::from(value).cast()).as_ref() }
    }
}

impl Drop for DynThinData {
    fn drop(&mut self) {
        let data_ptr = (&raw mut *self)
            .cast::<()>()
            .wrapping_byte_add(self.header.offset());

        unsafe { (self.header.drop)(data_ptr) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    const fn size_and_offset() {
        const {
            let x1 = SizeAndOffset::new(9, 3).unwrap();
            assert!(x1.size() == 9);
            assert!(x1.offset() == 3);

            let x2 = SizeAndOffset::new(123456, 15).unwrap();
            assert!(x2.size() == 123456);
            assert!(x2.offset() == 15);
        }
    }

    #[test]
    const fn data_wrapper_size_and_offset() {
        const {
            let x1 = ThinData::new(42);
            assert!(x1.header.size() == mem::size_of::<i32>());
            assert!(x1.header.offset() == mem::size_of::<ThinDataHeader>());
        }
    }

    #[test]
    fn zst() {
        struct Zst;

        let value = const { ThinData::new(Zst) };

        let dynamic = DynThinData::from_ref(&value);

        assert_eq!(dynamic.bytes.len(), 0);
        assert_eq!(mem::size_of_val(&value), mem::size_of_val(dynamic));
    }

    #[test]
    fn copy() {
        let value = const { ThinData::new(42) };
        let dynamic = DynThinData::from_ref(&value);

        assert_eq!(dynamic.bytes.len(), mem::size_of::<i32>());
        assert_eq!(mem::size_of_val(&value), mem::size_of_val(dynamic));
    }

    #[test]
    fn boxed_copy() {
        let value = ThinData::new(Box::new(42));
        let dynamic = DynThinData::from_ref(&value);

        assert_eq!(dynamic.bytes.len(), mem::size_of::<Box<i32>>());
        assert_eq!(mem::size_of_val(&value), mem::size_of_val(dynamic));
    }

    #[test]
    fn struct_with_drop_impl() {
        #[allow(unused)]
        struct Struct {
            values: Vec<i32>,
            value: Box<i32>,
        }

        let value = ThinData::new(Struct {
            values: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            value: Box::new(42),
        });

        let dynamic = DynThinData::from_ref(&value);

        assert_eq!(dynamic.bytes.len(), mem::size_of::<Struct>());
        assert_eq!(mem::size_of_val(&value), mem::size_of_val(dynamic));
    }

    #[test]
    fn big_alignment() {
        let value = const { ThinData::new(42_u128) };
        let dynamic = DynThinData::from_ref(&value);

        #[cfg(target_pointer_width = "64")]
        assert_eq!(dynamic.bytes.len(), mem::size_of::<u128>());

        assert_eq!(mem::size_of_val(&value), mem::size_of_val(dynamic));
    }

    #[test]
    fn zst_drop() {
        struct Zst;

        let value = Box::into_raw(Box::new(const { ThinData::new(Zst) }));
        let dynamic = unsafe { DynThinData::from_ptr(NonNull::new(value).unwrap().cast()) };

        drop(unsafe { Box::from_raw(dynamic.as_ptr()) });
    }

    #[test]
    fn copy_drop() {
        let value = Box::into_raw(Box::new(const { ThinData::new(42) }));
        let dynamic = unsafe { DynThinData::from_ptr(NonNull::new(value).unwrap().cast()) };

        drop(unsafe { Box::from_raw(dynamic.as_ptr()) });
    }

    #[test]
    fn boxed_copy_drop() {
        let value = Box::into_raw(Box::new(ThinData::new(Box::new(42))));
        let dynamic = unsafe { DynThinData::from_ptr(NonNull::new(value).unwrap().cast()) };

        drop(unsafe { Box::from_raw(dynamic.as_ptr()) });
    }

    #[test]
    fn struct_with_drop_impl_drop() {
        #[allow(unused)]
        struct Struct {
            values: Vec<i32>,
            value: Box<i32>,
        }

        let value = Box::into_raw(Box::new(ThinData::new(Struct {
            values: vec![1, 2, 3, 4, 5, 6, 7, 8],
            value: Box::new(42),
        })));

        let dynamic = unsafe { DynThinData::from_ptr(NonNull::new(value).unwrap().cast()) };

        drop(unsafe { Box::from_raw(dynamic.as_ptr()) });
    }
}
