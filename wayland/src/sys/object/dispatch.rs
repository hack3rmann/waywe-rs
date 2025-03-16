use crate::{
    object::{HasObjectType, WlObjectId},
    sys::{
        object_storage::WlObjectStorage,
        wire::{WlMessage, OpCode},
    },
};
use std::{
    ffi::{c_int, c_void, CStr}, panic, pin::Pin, process, ptr::NonNull, slice
};
use wayland_sys::{
    count_arguments_from_message_signature, wl_argument, wl_message, wl_proxy_get_id,
    wl_proxy_get_user_data,
};

pub trait Dispatch: 'static {
    fn dispatch(&mut self, _storage: Pin<&mut WlObjectStorage<'_>>, _message: WlMessage<'_>) {}
}
static_assertions::assert_obj_safe!(Dispatch);

pub(crate) type WlDispatchFn<T> = unsafe fn(&mut T, Pin<&mut WlObjectStorage<'_>>, WlMessage<'_>);

#[repr(C)]
pub(crate) struct WlDispatchData<T> {
    pub dispatch: WlDispatchFn<T>,
    pub storage: Option<NonNull<WlObjectStorage<'static>>>,
    pub data: T,
}

pub(crate) unsafe extern "C" fn dispatch_raw<T: HasObjectType>(
    _impl: *const c_void,
    proxy: *mut c_void,
    opcode: u32,
    message: *const wl_message,
    arguments: *mut wl_argument,
) -> c_int {
    tracing::info!(
        "libwayland event dispatch: {}::{}",
        T::OBJECT_TYPE.interface_name(),
        T::OBJECT_TYPE
            .event_name(opcode as OpCode)
            .unwrap_or("invalid-event"),
    );

    // NOTE(hack3rmann): to use `extern "Rust"` functions inside `extern "C"`
    // catching unwind is important to prevent UB from calling `panic()` in `extern "C"`
    panic::catch_unwind(|| {
        // Safety: `proxy` in libwayland dispatcher is always valid
        let id = unsafe { WlObjectId::try_from(wl_proxy_get_id(proxy)).unwrap_unchecked() };
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

        // # Safety
        //
        // - the storage pointer is pinned
        //   (see `WlObjectStorage::insert<T>(self: Pin<&mut Self>, object: WlObject<T>)`)
        // - here we have exclusive access to the storage
        //   (see `WlDisplay::dispatch(&self, _storage: Pin<&mut WlObjectStorage>)`)
        let storage = unsafe { Pin::new_unchecked(storage_ptr.as_mut()) };

        // Safety: an opcode provided by the libwayland backend is always valid (often really small)
        let opcode = unsafe { u16::try_from(opcode).unwrap_unchecked() };

        // # Safety
        //
        // - `message` points to a valid instance of `wl_message` (provided by libwayland)
        // - `message->signature` is a valid C-String (provided by libwayland)
        let signature = unsafe { CStr::from_ptr((*message).signature) };
        let n_arguments = count_arguments_from_message_signature(signature);

        // Safety: libwayland provides all arguments according to the signature of the event
        let arguments = unsafe { slice::from_raw_parts(arguments, n_arguments) };

        let message = WlMessage { opcode, arguments };

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
