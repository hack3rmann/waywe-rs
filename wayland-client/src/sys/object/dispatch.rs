use crate::{
    object::{HasObjectType, WlObjectId},
    sys::{
        object_storage::WlObjectStorage,
        wire::{OpCode, WlMessage},
    },
};
use std::{
    any::Any,
    cell::RefCell,
    ffi::{CStr, c_int, c_void},
    panic,
    pin::Pin,
    ptr::NonNull,
    slice,
};
use wayland_sys::{
    count_arguments_from_message_signature, wl_argument, wl_message, wl_proxy_get_id,
    wl_proxy_get_user_data,
};

/// # Note
///
/// State has to be sized because pointer to the state has to be thin.
pub trait State: Sized + 'static {}

/// An empty state.
pub struct NoState;

impl State for NoState {}

/// Types capable of dispatching the incoming events.
pub trait Dispatch: HasObjectType + 'static {
    type State: State;

    fn dispatch(
        &mut self,
        _state: Pin<&mut Self::State>,
        _storage: Pin<&mut WlObjectStorage<'_, Self::State>>,
        _message: WlMessage<'_>,
    ) {
    }
}

pub(crate) type WlDispatchFn<T, S> =
    fn(&mut T, Pin<&mut S>, Pin<&mut WlObjectStorage<'_, S>>, WlMessage<'_>);

#[repr(C)]
pub(crate) struct WlDispatchData<T, S: State> {
    pub dispatch: WlDispatchFn<T, S>,
    pub storage: Option<NonNull<WlObjectStorage<'static, S>>>,
    pub state: Option<NonNull<S>>,
    pub data: T,
}

thread_local! {
    pub(crate) static DISPATCHER_PANIC_CAUSE: RefCell<Option<Box<dyn Any + Send>>>
        = const { RefCell::new(None) };
}

pub(crate) fn handle_dispatch_raw_panic() {
    if let Some(error) = DISPATCHER_PANIC_CAUSE.with_borrow_mut(Option::take) {
        std::panic::resume_unwind(error);
    }
}

pub(crate) unsafe extern "C" fn dispatch_raw<T, S>(
    _impl: *const c_void,
    proxy: *mut c_void,
    opcode: u32,
    message: *const wl_message,
    arguments: *mut wl_argument,
) -> c_int
where
    T: HasObjectType,
    S: State,
{
    tracing::trace!(
        interface = T::OBJECT_TYPE.interface_name(),
        event = T::OBJECT_TYPE
            .event_name(opcode as OpCode)
            .unwrap_or("invalid_event"),
        "dispatch_raw",
    );

    // NOTE(hack3rmann): to use `extern "Rust"` functions inside `extern "C"`
    // catching unwind is important to prevent UB from calling `panic()` in `extern "C"`
    // and continuing stack unwind outside of `extern "C"` context
    panic::catch_unwind(|| {
        // All code below relies on the fact that in previous dispatch
        // invocation the object storage has released object data being used
        // in this dispatcher.
        if DISPATCHER_PANIC_CAUSE.with_borrow(Option::is_some) {
            return -1;
        }

        // Safety: `proxy` in libwayland dispatcher is always valid
        let id = unsafe { WlObjectId::try_from(wl_proxy_get_id(proxy)).unwrap_unchecked() };
        let data = unsafe { wl_proxy_get_user_data(proxy) }.cast::<WlDispatchData<T, S>>();

        // # Safety
        //
        // - `data` points to a valid box-allocated instance of `WlDispatchData`
        // - `data` only being used in dispatcher, libwayland provides exclusive access to the data
        let Some(data) = (unsafe { data.as_mut() }) else {
            tracing::error!("no user data is set");
            return -1;
        };

        let Some(mut storage_ptr) = data.storage.map(|p| p.cast::<WlObjectStorage<S>>()) else {
            tracing::error!("no pointer to `WlObjectStorage` is set");
            return -1;
        };

        let Some(mut state_ptr) = data.state else {
            tracing::error!("no pointer to state is set");
            return -1;
        };

        // # Safety
        //
        // - the storage pointer is pinned
        //   (see `WlObjectStorage::insert<T>(self: Pin<&mut Self>, object: WlObject<T>)`)
        // - here we have exclusive access to the storage
        //   (see `WlDisplay::dispatch`)
        let storage = unsafe { Pin::new_unchecked(storage_ptr.as_mut()) };

        // # Safety
        //
        // - the state pointer is pinned
        //   (see `WlObjectStorage::insert<T>(self: Pin<&mut Self>, object: WlObject<T>)`)
        // - here we have exclusive access to the state
        //   (see `WlDisplay::dispatch`)
        let state = unsafe { Pin::new_unchecked(state_ptr.as_mut()) };

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

        if let Err(err) = storage.with_object_data_acquired(id, |storage| {
            (data.dispatch)(&mut data.data, state, storage, message);
        }) {
            tracing::error!("failed to acquire the object's data: {err}");
            return -1;
        }

        0
    })
    .unwrap_or_else(|cause| {
        tracing::error!("panic in {}::dispatch_raw(..)", module_path!());
        DISPATCHER_PANIC_CAUSE.with_borrow_mut(|e| _ = e.insert(cause));
        -1
    })
}
