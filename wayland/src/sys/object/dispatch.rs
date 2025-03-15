use crate::{
    object::ObjectId,
    sys::{object_storage::WlObjectStorage, wire::Message},
};
use std::{
    ffi::{CStr, c_int, c_void},
    pin::Pin,
    process,
    ptr::NonNull,
    slice,
};
use wayland_sys::{
    count_arguments_from_message_signature, wl_argument, wl_message, wl_proxy_get_id,
    wl_proxy_get_user_data,
};

pub trait Dispatch: 'static {
    fn dispatch(&mut self, _storage: Pin<&mut WlObjectStorage<'_>>, _message: Message<'_>) {}
}
static_assertions::assert_obj_safe!(Dispatch);

pub(crate) type WlDispatchFn<T> = unsafe fn(&mut T, Pin<&mut WlObjectStorage<'_>>, Message<'_>);

#[repr(C)]
pub(crate) struct WlDispatchData<T> {
    pub dispatch: WlDispatchFn<T>,
    pub storage: Option<NonNull<WlObjectStorage<'static>>>,
    pub data: T,
}

pub(crate) unsafe extern "C" fn dispatch_raw<T>(
    _impl: *const c_void,
    proxy: *mut c_void,
    opcode: u32,
    message: *const wl_message,
    arguments: *mut wl_argument,
) -> c_int {
    std::panic::catch_unwind(|| {
        // Safety: `proxy` in libwayland dispatcher is always valid
        let id = unsafe { ObjectId::try_from(wl_proxy_get_id(proxy)).unwrap_unchecked() };
        let data = unsafe { wl_proxy_get_user_data(proxy) }.cast::<WlDispatchData<T>>();

        // # Safety
        //
        // - `data` points to a valid box-allocated instance of `WlDispatchData`
        // - `data` only being used in dispatcher, libwayland provides exclusive access to the data
        let Some(data) = (unsafe { data.as_mut() }) else {
            tracing::error!("no user data is set");
            return -1;
        };

        let Some(mut storage_ptr) = data.storage.map(|p| p.cast::<WlObjectStorage>()) else {
            tracing::error!("no pointer to `WlObjectStorage` is set");
            return -1;
        };

        let storage = unsafe { Pin::new_unchecked(storage_ptr.as_mut()) };

        let Ok(opcode) = u16::try_from(opcode) else {
            tracing::error!("invalid opcode {opcode}");
            return -1;
        };

        // # Safety
        //
        // - `message` points to a valid instance of `wl_message` (provided by libwayland)
        // - `message->signature` is a valid C-String (provided by libwayland)
        let signature = unsafe { CStr::from_ptr((*message).signature) };
        let n_arguments = count_arguments_from_message_signature(signature);

        // Safety: libwayland provides all arguments according to the signature of the event
        let arguments = unsafe { slice::from_raw_parts(arguments, n_arguments) };

        let message = Message { opcode, arguments };

        if let Err(err) = storage.with_object_data_acquired(id, |storage| unsafe {
            (data.dispatch)(&mut data.data, storage, message);
        }) {
            tracing::error!("failed to acquire the object's data: {err}");
            return -1;
        }

        0
    })
    .unwrap_or_else(|_| {
        tracing::error!("panic in wl_dispatcher_func_t");
        process::abort();
    })
}
