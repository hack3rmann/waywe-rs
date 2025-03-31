use std::{
    mem::{self, MaybeUninit},
    ptr::NonNull,
};

#[derive(Clone, Copy)]
pub(crate) struct SizeAndOffset(pub(crate) usize);

impl SizeAndOffset {
    pub(crate) const N_OFFSET_BITS: usize = 4;
    pub(crate) const MAX_OFFSET: usize = (1 << Self::N_OFFSET_BITS) - 1;

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

#[derive(Clone, Copy)]
pub(crate) struct DataWrapperHeader {
    pub(crate) drop: unsafe fn(*mut ()),
    // TODO(hack3rmann): we can pack both `size` and `offset` into one `u16` type
    pub(crate) size_and_offset: SizeAndOffset,
}

impl DataWrapperHeader {
    pub(crate) const OFFSET_SHIFT: usize = mem::size_of::<DataWrapperHeader>();

    pub(crate) const fn of<T: 'static>() -> Self {
        let drop = |ptr: *mut ()| unsafe {
            ptr.cast::<T>().drop_in_place();
        };

        Self {
            drop,
            size_and_offset: const {
                SizeAndOffset::new(
                    mem::size_of::<T>(),
                    mem::offset_of!(DataWrapper<T>, data) - Self::OFFSET_SHIFT,
                )
                .unwrap()
            },
        }
    }

    pub(crate) const fn size(self) -> usize {
        self.size_and_offset.size()
    }

    pub(crate) const fn offset(self) -> usize {
        self.size_and_offset.offset() + Self::OFFSET_SHIFT
    }
}

#[repr(C)]
pub(crate) struct DataWrapper<T: 'static> {
    pub(crate) header: DataWrapperHeader,
    pub(crate) data: T,
}

impl<T: 'static> DataWrapper<T> {
    pub(crate) const fn new(data: T) -> Self {
        Self {
            header: const { DataWrapperHeader::of::<T>() },
            data,
        }
    }
}

#[repr(C)]
pub(crate) struct DynDataWrapper {
    pub(crate) header: DataWrapperHeader,
    pub(crate) bytes: [MaybeUninit<u8>],
}

impl DynDataWrapper {
    pub(crate) unsafe fn from_ptr(ptr: NonNull<()>) -> NonNull<Self> {
        let header = unsafe { ptr.cast::<DataWrapperHeader>().as_ref() };

        let n_align_bytese = header.offset() - mem::size_of::<DataWrapperHeader>();
        let byte_slice_size = header.size() + n_align_bytese;

        // HACK(hack3rmann): constructing fat pointer like this is non-const
        unsafe { mem::transmute::<(NonNull<()>, usize), NonNull<Self>>((ptr, byte_slice_size)) }
    }

    pub(crate) fn from_ref<T: 'static>(value: &DataWrapper<T>) -> &Self {
        unsafe { Self::from_ptr(NonNull::from(value).cast()).as_ref() }
    }
}

impl Drop for DynDataWrapper {
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
    fn size_and_offset() {
        const X1: SizeAndOffset = SizeAndOffset::new(9, 3).unwrap();
        const { assert!(X1.size() == 9) };
        const { assert!(X1.offset() == 3) };

        const X2: SizeAndOffset = SizeAndOffset::new(123456, 15).unwrap();
        const { assert!(X2.size() == 123456) };
        const { assert!(X2.offset() == 15) };
    }

    #[test]
    fn data_wrapper_size_and_offset() {
        const X1: DataWrapper<i32> = DataWrapper::new(42);
        const { assert!(X1.header.size() == mem::size_of::<i32>()) };
        const { assert!(X1.header.offset() == mem::size_of::<DataWrapperHeader>()) };
    }

    #[test]
    fn zst() {
        struct Zst;

        let value = const { DataWrapper::new(Zst) };

        let dynamic = DynDataWrapper::from_ref(&value);

        assert_eq!(dynamic.bytes.len(), 0);
        assert_eq!(mem::size_of_val(&value), mem::size_of_val(dynamic));
    }

    #[test]
    fn copy() {
        let value = const { DataWrapper::new(42) };
        let dynamic = DynDataWrapper::from_ref(&value);

        assert_eq!(dynamic.bytes.len(), mem::size_of::<i32>());
        assert_eq!(mem::size_of_val(&value), mem::size_of_val(dynamic));
    }

    #[test]
    fn boxed_copy() {
        let value = DataWrapper::new(Box::new(42));
        let dynamic = DynDataWrapper::from_ref(&value);

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

        let value = DataWrapper::new(Struct {
            values: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            value: Box::new(42),
        });

        let dynamic = DynDataWrapper::from_ref(&value);

        assert_eq!(dynamic.bytes.len(), mem::size_of::<Struct>());
        assert_eq!(mem::size_of_val(&value), mem::size_of_val(dynamic));
    }

    #[test]
    fn big_alignment() {
        let value = const { DataWrapper::new(42_u128) };
        let dynamic = DynDataWrapper::from_ref(&value);

        #[cfg(target_pointer_width = "64")]
        assert_eq!(dynamic.bytes.len(), mem::size_of::<u128>());

        assert_eq!(mem::size_of_val(&value), mem::size_of_val(dynamic));
    }

    #[test]
    fn zst_drop() {
        struct Zst;

        let value = Box::into_raw(Box::new(const { DataWrapper::new(Zst) }));
        let dynamic = unsafe { DynDataWrapper::from_ptr(NonNull::new(value).unwrap().cast()) };

        drop(unsafe { Box::from_raw(dynamic.as_ptr()) });
    }

    #[test]
    fn copy_drop() {
        let value = Box::into_raw(Box::new(const { DataWrapper::new(42) }));
        let dynamic = unsafe { DynDataWrapper::from_ptr(NonNull::new(value).unwrap().cast()) };

        drop(unsafe { Box::from_raw(dynamic.as_ptr()) });
    }

    #[test]
    fn boxed_copy_drop() {
        let value = Box::into_raw(Box::new(DataWrapper::new(Box::new(42))));
        let dynamic = unsafe { DynDataWrapper::from_ptr(NonNull::new(value).unwrap().cast()) };

        drop(unsafe { Box::from_raw(dynamic.as_ptr()) });
    }

    #[test]
    fn struct_with_drop_impl_drop() {
        #[allow(unused)]
        struct Struct {
            values: Vec<i32>,
            value: Box<i32>,
        }

        let value = Box::into_raw(Box::new(DataWrapper::new(Struct {
            values: vec![1, 2, 3, 4, 5, 6, 7, 8],
            value: Box::new(42),
        })));

        let dynamic = unsafe { DynDataWrapper::from_ptr(NonNull::new(value).unwrap().cast()) };

        drop(unsafe { Box::from_raw(dynamic.as_ptr()) });
    }
}
