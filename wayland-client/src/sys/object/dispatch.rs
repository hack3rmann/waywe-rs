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
    ffi::{c_int, c_void},
    mem, panic,
    ptr::NonNull,
    slice,
};
use wayland_sys::{
    count_arguments_from_message_signature_raw, wl_argument, wl_message, wl_proxy_get_id,
    wl_proxy_get_user_data,
};

/// # Note
///
/// State has to be sized because pointer to the state has to be thin.
pub trait State: Sync + Sized + 'static {}

/// An empty state.
pub struct NoState;

impl State for NoState {}

/// Types capable of dispatching the incoming events.
pub trait Dispatch: HasObjectType + 'static {
    type State: State;

    /// A small optimization for dispatch handlers.
    ///
    /// A hint to [`wayland_client`](crate) implementation about
    /// dispatcher implementation. If some contraints on `Self`
    /// have met it will remove memory allocation associated
    /// with creation of [`WlObject`](super::WlObject) and usage of `Self`
    /// entierly. Also, it will ignore almost all checks being performed on
    /// this data in the raw dispatcher.
    const ALLOW_EMPTY_DISPATCH: bool = false;

    fn dispatch(
        &mut self,
        _state: &Self::State,
        _storage: &mut WlObjectStorage<'_, Self::State>,
        _message: WlMessage<'_>,
    ) {
        // do nothing
    }
}

/// Failes compilation if dispatch implementation for `T` is not empty
///
/// # Example
///
/// ```rust
/// # use wayland_client::{
/// #     HasObjectType, WlProxy, FromProxy, Dispatch,
/// #     NoState, WlObjectType, WlMessage, WlObjectStorage,
/// #     assert_dispatch_is_empty,
/// # };
/// pub struct WlSurface;
///
/// # impl HasObjectType for WlSurface {
/// #     const OBJECT_TYPE: WlObjectType = WlObjectType::Surface;
/// # }
///
/// impl Dispatch for WlSurface {
///     type State = NoState;
///     const ALLOW_EMPTY_DISPATCH: bool = true;
/// }
///
/// assert_dispatch_is_empty!(WlSurface);
/// ```
///
/// This fails to compile:
///
/// ```rust,compile_fail
/// # use wayland_client::{
/// #     HasObjectType, WlProxy, FromProxy, Dispatch,
/// #     NoState, WlObjectType, WlMessage, WlObjectStorage,
/// #     assert_dispatch_is_empty,
/// # };
/// // Non-ZST with Drop implementation
/// pub struct WlCompositor(pub Vec<()>);
///
/// # impl HasObjectType for WlCompositor {
/// #     const OBJECT_TYPE: WlObjectType = WlObjectType::Compositor;
/// # }
///
/// impl Dispatch for WlCompositor {
///     type State = NoState;
///
///     // Allowing empty dispatch does not mean that it will use it
///     const ALLOW_EMPTY_DISPATCH: bool = true;
/// }
///
/// assert_dispatch_is_empty!(WlCompositor);
/// ```
#[macro_export]
macro_rules! assert_dispatch_is_empty {
    ( $T:ty ) => {
        const _: () = const {
            assert!(
                $crate::sys::object::dispatch::is_empty_dispatch_data_allowed::<$T>(),
                "dispatch is not empty",
            );
        };
    };
}

/// Checks that dispatch implementation for `T` is empty
pub const fn is_empty_dispatch_data_allowed<T: Dispatch>() -> bool {
    // NOTE(hack3rmann):
    // - empty dispatcher should be allowed
    // - `T` should be ZST to allow `WlObject` create references to it
    // - dropping `T` should has no side effects, because constructing
    //   `WlObject` may drop one inconsistently
    T::ALLOW_EMPTY_DISPATCH && mem::size_of::<T>() == 0 && !mem::needs_drop::<T>()
}

pub(crate) type WlDispatchFn<T, S> = fn(&mut T, &S, &mut WlObjectStorage<'_, S>, WlMessage<'_>);

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

pub(crate) fn handle_panic() {
    if let Some(error) = DISPATCHER_PANIC_CAUSE.with_borrow_mut(Option::take) {
        panic::resume_unwind(error);
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
    // `dispatch_raw` may be called several times after the last panic
    if DISPATCHER_PANIC_CAUSE.with_borrow(Option::is_some) {
        return -1;
    }

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
        // Safety: `proxy` in libwayland dispatcher is always valid
        let data = unsafe { wl_proxy_get_user_data(proxy) }.cast::<WlDispatchData<T, S>>();

        // # Safety
        //
        // - `data` points to a valid box-allocated instance of `WlDispatchData`
        // - `data` only being used in dispatcher, libwayland provides exclusive access to the data
        let Some(data) = (unsafe { data.as_mut() }) else {
            tracing::error!("no data pointer is set");
            return -1;
        };

        // Safety: `proxy` in libwayland dispatcher is always valid
        let id = unsafe { WlObjectId::try_from(wl_proxy_get_id(proxy)).unwrap_unchecked() };

        let Some(mut storage_ptr) = data.storage.map(|p| p.cast::<WlObjectStorage<S>>()) else {
            tracing::error!("no pointer to `WlObjectStorage` is set");
            return -1;
        };

        let Some(mut state_ptr) = data.state else {
            tracing::error!("no pointer to state is set");
            return -1;
        };

        // Safety: we have exclusive access to the storage (see `WlDisplay::dispatch`)
        let storage = unsafe { storage_ptr.as_mut() };

        // Safety: we have exclusive access to the state (see `WlDisplay::dispatch`)
        let state = unsafe { state_ptr.as_mut() };

        // Safety: an opcode provided by the libwayland backend is always valid (often really small)
        let opcode = unsafe { u16::try_from(opcode).unwrap_unchecked() };

        // # Safety
        //
        // - `message` points to a valid instance of `wl_message` (provided by libwayland)
        // - `message->signature` is a valid C-String (provided by libwayland)
        let n_arguments =
            unsafe { count_arguments_from_message_signature_raw((*message).signature) };

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
