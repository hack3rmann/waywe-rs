//! Collection of tools for implementing wayland-clients from scratch.
//!
//! # Overview
//!
//! This crate provides *blazingly-fast* and *safe* functionality for implementing
//! Wayland-Clients from scratch utilizing Rust's lifetimes and type system to
//! make as few allocations and other kinds of dynamic operations as possible.
//!
//! ## Dispatch
//!
//! This crate is built around the [`Dispatch`] trait. All types of Wayland
//! objects should implement this trait to be capable of dispatching Wayland events.
//! Having event loop idea in mind, [`Dispatch::dispatch`] also references
//! global state by means of a struct that implements the [`State`] trait. This trait
//! is automatically implemented for all [`Sync + 'static`](std::marker::Sync) types.
//!
//! ## Display
//!
//! The main object this crate provides is [`WlDisplay`]. It can be used
//! to create [`WlEventQueue`]s, [`WlRegistry`]s and to dispatch all events
//! from a given queue at once (see [`WlDisplay::roundtrip`]).
//!
//! # Initial setup
//!
//! In order to communicate with Wayland, [`WlDisplay`] must be created.
//! [`WlDisplay`] will dispatch all events on a given [`WlEventQueue`].
//! Display has the default queue to dispatch events called 'the main queue'.
//! Both *state* and *event queue* should be pinned to be useful later.
//! Long story short, `libwayland` implementation passes all the pointers around,
//! therefore ensuring safety implies pinning all the pointers (e.g. on stack).
//!
//! ```rust
//! use wayland_client::{WlStackMessageBuffer, WlDisplay};
//! use std::pin::pin;
//!
//! struct ClientState {}
//!
//! // State has to be pinned
//! let state = pin!(ClientState {});
//!
//! // Connect to Wayland
//! let display = WlDisplay::connect(state.as_ref()).unwrap();
//!
//! // Take the main display event queue, pin it
//! let mut queue = pin!(display.take_main_queue().unwrap());
//! ```
//!
//! # Implementing the [`Dispatch`] trait
//!
//! The core [`wayland_client`](crate) trait. Allows the implementor type
//! to proccess the events incoming to this object.
//!
//! ```rust
//! # struct ClientState;
//! use wayland_client::{
//!     HasObjectType, WlObjectType, Dispatch, WlObjectStorage, WlMessage,
//!     interface::WlShmFormatEvent,
//! };
//!
//! // Almost all objects are defined by user.
//! struct Shm;
//!
//! impl HasObjectType for Shm {
//!     // Type of this object
//!     const OBJECT_TYPE: WlObjectType = WlObjectType::Shm;
//! }
//!
//! impl Dispatch for Shm {
//!     type State = ClientState;
//!
//!     fn dispatch(
//!         &mut self,
//!         // Global client state to be used
//!         _state: &Self::State,
//!         // Storage this object belongs to
//!         _storage: &mut WlObjectStorage<Self::State>,
//!         // Message to be dispatched
//!         message: WlMessage<'_>,
//!     ) {
//!         // Parse the event
//!         if let Some(_event) = message.as_event::<WlShmFormatEvent>() {
//!             // Process the event
//!         }
//!     }
//! }
//! ```
//!
//! # Empty [`Dispatch`] optimization
//!
//! Implementing the [`Dispatch`] trait goes with an overhead of checking several
//! constraints at runtime, allocation of assocciated data and catching Rust panics.
//! If you want an empty [`Dispatch`] implementation, there is a nice optimization.
//! Setting the [`Dispatch::ALLOW_EMPTY_DISPATCH`] const to `true` hints [`wayland_client`](crate)
//! that it can leave dispatch implementation empty, if some constraints are met.
//!
//! This optimization requires implementor type of the [`Dispatch`] trait
//!
//! 1. to be ZST
//! 2. not to implement the [`Drop`] trait
//! 3. [`Dispatch::ALLOW_EMPTY_DISPATCH`] set to `true`
//!
//! ```rust
//! # use wayland_client::{HasObjectType, WlObjectType, Dispatch};
//! # struct ClientState;
//! # impl HasObjectType for Compositor {
//! #     const OBJECT_TYPE: WlObjectType = WlObjectType::Compositor;
//! # }
//! use wayland_client::assert_dispatch_is_empty;
//!
//! // Dispatcher should be zero-sized without drop implementation.
//! struct Compositor;
//!
//! impl Dispatch for Compositor {
//!     type State = ClientState;
//!     // This flag should be set to `true`
//!     const ALLOW_EMPTY_DISPATCH: bool = true;
//! }
//!
//! // Make sure this type implements empty dispatcher
//! assert_dispatch_is_empty!(Compositor);
//! ```
//!
//! [`wayland_client`](crate) also provides a macro named [`assert_dispatch_is_empty`]
//! to be extra sure about dispatch implementation being empty.
//!
//! ```rust,compile_fail
//! # use wayland_client::{HasObjectType, WlObjectType, Dispatch};
//! # struct ClientState;
//! # impl HasObjectType for Compositor {
//! #     const OBJECT_TYPE: WlObjectType = WlObjectType::Compositor;
//! # }
//! # struct Compositor;
//! use wayland_client::assert_dispatch_is_empty;
//!
//! // Type implements the `Drop` trait
//! impl Drop for Compositor {
//!     fn drop(&mut self) {}
//! }
//!
//! impl Dispatch for Compositor {
//!     type State = ClientState;
//!     // Even if this flag is set to `true` dispatch implementation is not empty
//!     const ALLOW_EMPTY_DISPATCH: bool = true;
//! }
//!
//! assert_dispatch_is_empty!(Compositor);
//! ```
//!
//! # Binding to globals
//!
//! In order to build a request message, a kind of `[`WlMessageBuffer`] should be created.
//! [`wayland_client`](crate) provides implementation of 3 different message buffers:
//!
//! 1. [`WlVecMessageBuffer`] using [`Vec`] as a backend buffer
//! 2. [`WlSmallVecMessageBuffer`] using [`SmallVec`](smallvec::SmallVec) as a backend buffer
//! 3. [`WlStackMessageBuffer`] implementation confined to the stack.
//!
//! Binding to globals requires [`WlRegistry`] to be created and initialized via
//! the event dispatch. Event dispatch happens on the given event queue when
//! [`WlDisplay::roundtrip`] is called on this queue.
//!
//! ```rust
//! # use wayland_client::{HasObjectType, Dispatch, WlObjectType, WlStackMessageBuffer, WlDisplay};
//! # use std::pin::pin;
//! # struct ClientState {}
//! # impl HasObjectType for Compositor {
//! #     const OBJECT_TYPE: WlObjectType = WlObjectType::Compositor;
//! # }
//! # struct Compositor;
//! # impl Dispatch for Compositor {
//! #     type State = ClientState;
//! #     const ALLOW_EMPTY_DISPATCH: bool = true;
//! # }
//! # let state = pin!(ClientState {});
//! # let display = WlDisplay::connect(state.as_ref()).unwrap();
//! # let mut queue = pin!(display.take_main_queue().unwrap());
//! use wayland_client::{FromProxy, WlProxy};
//!
//! // Globals being bound through a call to `.bind` on `WlRegistry`
//! // should implement the `FromProxy` trait
//! impl FromProxy for Compositor {
//!     // In our case, `Self` is ZST
//!     fn from_proxy(_proxy: &WlProxy) -> Self { Self }
//! }
//!
//! // Message buffer for message building
//! let mut buf = WlStackMessageBuffer::new();
//!
//! // Request Wayland to create a registry. The registry is empty initially.
//! // `registry` itself is essentially a lightweight handle.
//! let registry = display.create_registry(&mut buf, queue.as_mut().storage_mut());
//!
//! // We should wait the registry to be filled.
//! display.roundtrip(queue.as_mut(), state.as_ref());
//!
//! // If Wayland-Server has this global present, it creates
//! // a new object on the storage of the main `WlEventQueue` `queue`
//! // and returns a weak handle to it
//! let compositor = registry
//!     .bind::<Compositor>(&mut buf, queue.as_mut().storage_mut())
//!     .unwrap();
//!
//! // `Compositor` can be accessed like this:
//! let _value: &Compositor = queue.as_ref().storage().object(compositor);
//! ```
//!
//! # Making requests
//!
//! [`wayland_client`](crate) provides a way to make a request on a particular
//! object via calls to [`WlObjectHandle::request`] and [`WlObjectHandle::create_object`].
//! It is a general and type-safe way to create wayland objects and make any request.
//! As a safety condideration the program won't compile if a user mistypes a dispatcher
//! name or makes a request with different interface different from the request's parental one.
//!
//! ```rust
//! # use wayland_client::{
//! #     HasObjectType, Dispatch, WlObjectType, WlStackMessageBuffer,
//! #     WlDisplay, FromProxy, WlProxy,
//! # };
//! # use std::pin::pin;
//! # struct ClientState {}
//! # impl HasObjectType for Compositor {
//! #     const OBJECT_TYPE: WlObjectType = WlObjectType::Compositor;
//! # }
//! # struct Compositor;
//! # impl Dispatch for Compositor {
//! #     type State = ClientState;
//! #     const ALLOW_EMPTY_DISPATCH: bool = true;
//! # }
//! # let mut buf = WlStackMessageBuffer::new();
//! # let state = pin!(ClientState {});
//! # let display = WlDisplay::connect(state.as_ref()).unwrap();
//! # let mut queue = pin!(display.take_main_queue().unwrap());
//! # impl FromProxy for Compositor {
//! #     fn from_proxy(_proxy: &WlProxy) -> Self { Self }
//! # }
//! # let registry = display.create_registry(&mut buf, queue.as_mut().storage_mut());
//! # display.roundtrip(queue.as_mut(), state.as_ref());
//! # let compositor = registry
//! #     .bind::<Compositor>(&mut buf, queue.as_mut().storage_mut())
//! #     .unwrap();
//! use wayland_client::{
//!     WlObjectHandle,
//!     interface::{WlCompositorCreateSurfaceRequest, WlSurfaceDestroyRequest},
//! };
//!
//! struct Surface;
//!
//! impl HasObjectType for Surface {
//!     const OBJECT_TYPE: WlObjectType = WlObjectType::Surface;
//! }
//!
//! impl Dispatch for Surface {
//!     # type State = ClientState;
//!     # const ALLOW_EMPTY_DISPATCH: bool = true;
//!     // ...
//! }
//!
//! impl FromProxy for Surface {
//!     # fn from_proxy(_: &WlProxy) -> Self { Self }
//!     // ...
//! }
//!
//! // Object creating requests can be done like this.
//! let surface: WlObjectHandle<Surface> = compositor.create_object(
//!     &mut buf,
//!     queue.as_mut().storage_mut(),
//!     // Request-specific data
//!     WlCompositorCreateSurfaceRequest,
//! );
//!
//! // Regular requests
//! surface.request(
//!     &mut buf,
//!     &queue.as_ref().storage(),
//!     WlSurfaceDestroyRequest,
//! );
//!
//! // Surface object can then be destroyed like this
//! queue.as_mut().storage_mut().release(surface).unwrap();
//! ```

pub(crate) mod init;
pub mod interface;
pub mod object;
pub mod sys;

#[doc(hidden)]
pub use {
    interface::WlObjectType,
    object::{HasObjectType, WlObjectId},
    sys::{
        display::{DisplayConnectError, DisplayConnectToFdError, WlDisplay},
        object::{
            FromProxy,
            dispatch::{Dispatch, NoState, State},
            event_queue::{CreateQueueError, WlEventQueue},
            registry::WlRegistry,
            {NonZstError, WlObject, WlObjectHandle},
        },
        object_storage::{NoEntryError, ObjectDataAcquireError, WlObjectStorage},
        protocol,
        proxy::WlProxy,
        wire::{
            OpCode, WlMessage, WlMessageBuffer, WlMessageBuilder, WlMessageBuilderHeaderless,
            WlMessageReader, WlStackMessageBuffer, WlVecMessageBuffer,
        },
    },
};

#[cfg(feature = "smallvec")]
pub use sys::wire::WlSmallVecMessageBuffer;

#[doc(hidden)]
pub(crate) use wayland_sys as ffi;
