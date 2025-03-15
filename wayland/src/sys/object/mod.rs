pub mod buffer;
pub mod callback;
pub mod compositor;
pub mod dispatch;
pub mod output;
pub mod region;
pub mod registry;
pub mod shm;
pub mod shm_pool;
pub mod surface;
pub mod viewport;
pub mod viewporter;
pub mod zwlr_layer_shell_v1;
pub mod zwlr_layer_surface_v1;

use super::{HasObjectType, object_storage::WlObjectStorage, proxy::WlProxy, wire::MessageBuffer};
use crate::{
    interface::{ObjectParent, Request},
    object::ObjectId,
};
use dispatch::{Dispatch, WlDispatchData, dispatch_raw};
use std::{
    any::{self, TypeId},
    fmt, hash,
    marker::PhantomData,
    mem::{self, MaybeUninit, offset_of},
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr::{self, NonNull},
};
use wayland_sys::wl_proxy_add_dispatcher;

/// A trait used to construct newly created object.
pub trait FromProxy: Sized {
    fn from_proxy(proxy: &WlProxy) -> Self;
}

/// Lightweight object handle with assocciated data type
pub struct WlObjectHandle<T> {
    pub(crate) id: ObjectId,
    pub(crate) _p: PhantomData<T>,
}

impl<T> WlObjectHandle<T> {
    /// Creates a handle to [`WlObject<T>`]
    pub const fn new(id: ObjectId) -> Self {
        Self {
            id,
            _p: PhantomData,
        }
    }

    pub fn request<'r, R>(self, buf: &mut impl MessageBuffer, storage: &WlObjectStorage, request: R)
    where
        T: Dispatch + HasObjectType,
        R: Request<'r>,
    {
        const {
            assert!(
                T::OBJECT_TYPE as u32 == R::OBJECT_TYPE as u32,
                "request's parent interface should match the self type one's"
            );

            assert!(
                R::OUTGOING_INTERFACE.is_none(),
                "request's outgoing interface should be set to None",
            )
        };

        let proxy = unsafe { request.send(buf, storage, storage.get_proxy(self.id).unwrap()) };
        debug_assert!(proxy.is_none());
    }

    pub fn create_object<'r, R>(
        self,
        buf: &mut impl MessageBuffer,
        storage: Pin<&mut WlObjectStorage>,
        request: R,
    ) -> WlObjectHandle<R::Child>
    where
        R: Request<'r> + ObjectParent,
        R::Child: Dispatch + HasObjectType + FromProxy,
        T: Dispatch + HasObjectType,
    {
        const {
            assert!(
                T::OBJECT_TYPE as u32 == R::OBJECT_TYPE as u32,
                "request's parent interface should match the self type one's"
            );

            match R::OUTGOING_INTERFACE {
                Some(object_type) => assert!(
                    object_type as u32 == R::Child::OBJECT_TYPE as u32,
                    "request's outgoing interface should match the return type one's"
                ),
                None => panic!("the request should have outgoing interface set to Some"),
            }
        };

        let proxy = unsafe {
            request
                .send(
                    buf,
                    storage.as_ref().get_ref(),
                    storage.get_proxy(self.id).unwrap(),
                )
                .unwrap()
        };

        let data = R::Child::from_proxy(&proxy);
        storage.insert(WlObject::new(proxy, data))
    }
}

impl<T> Default for WlObjectHandle<T> {
    fn default() -> Self {
        Self::new(ObjectId::default())
    }
}

impl<T> hash::Hash for WlObjectHandle<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        hash::Hash::hash(&self.id, state);
    }
}

impl<T> Clone for WlObjectHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for WlObjectHandle<T> {}

impl<T> fmt::Debug for WlObjectHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(any::type_name::<Self>())
            .field("id", &self.id)
            .finish()
    }
}

impl<T> PartialEq for WlObjectHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for WlObjectHandle<T> {}

#[derive(Clone, Debug, PartialEq, Copy)]
pub(crate) struct TypeInfo {
    pub(crate) id: TypeId,
    pub(crate) drop: unsafe fn(*mut ()),
}

impl TypeInfo {
    pub(crate) fn of<T: 'static>() -> TypeInfo {
        TypeInfo {
            id: TypeId::of::<T>(),
            drop: |ptr: *mut ()| unsafe {
                ptr.cast::<T>().drop_in_place();
            },
        }
    }
}

#[repr(C)]
pub(crate) struct WlDynObject {
    pub(crate) proxy: WlProxy,
    pub(crate) type_info: TypeInfo,
}

impl WlDynObject {
    pub(crate) fn downcast_ref<T: 'static>(&self) -> Option<&WlObject<T>> {
        // # Safety
        //
        // - `WlDynObject` and `WlObject<T>` have the same header - `WlProxy`
        // - both structs are `repr(C)`
        (self.type_info.id == TypeId::of::<T>())
            .then_some(unsafe { mem::transmute::<&WlDynObject, &WlObject<T>>(self) })
    }

    pub(crate) fn downcast_mut<T: 'static>(&mut self) -> Option<&mut WlObject<T>> {
        // # Safety
        //
        // - `WlDynObject` and `WlObject<T>` have the same header - `WlProxy`
        // - both structs are `repr(C)`
        (self.type_info.id == TypeId::of::<T>())
            .then_some(unsafe { mem::transmute::<&mut WlDynObject, &mut WlObject<T>>(self) })
    }
}

impl Drop for WlDynObject {
    fn drop(&mut self) {
        // Safety: `self.proxy` is a valid object produced by libwayland
        let user_data = self.proxy.get_user_data();

        let data_ptr = user_data
            .wrapping_byte_add(offset_of!(WlDispatchData<()>, data))
            .cast::<()>();

        // # Safety
        //
        // - `data_ptr` points to a valid location of `T`
        // - `drop` called once
        unsafe { (self.type_info.drop)(data_ptr) }
    }
}

impl fmt::Debug for WlDynObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(any::type_name::<Self>())
            .field("proxy", &self.proxy)
            .finish_non_exhaustive()
    }
}

#[repr(C)]
pub struct WlObject<T> {
    pub(crate) proxy: WlProxy,
    pub(crate) _p: PhantomData<T>,
}

impl<T: Dispatch> WlObject<T> {
    pub fn new(proxy: WlProxy, data: T) -> Self {
        let dispatch_data = Box::new(WlDispatchData {
            dispatch: T::dispatch,
            storage: None,
            data,
        });

        let dispatch_data_ptr = Box::into_raw(dispatch_data);

        // Safety: `proxy` is a valid object provided by libwayland
        let result = unsafe {
            wl_proxy_add_dispatcher(
                proxy.as_raw().as_ptr(),
                dispatch_raw::<T>,
                ptr::null(),
                dispatch_data_ptr.cast(),
            )
        };

        if -1 == result {
            // Safety: `WlObject` have not constructed yet
            // therefore we should take care of the `Box` ourselves
            drop(unsafe { Box::from_raw(dispatch_data_ptr) });
            panic!("`wl_proxy_add_dispatcher` failed");
        }

        Self {
            proxy,
            _p: PhantomData,
        }
    }

    pub fn write_storage_location(&mut self, mut storage: Pin<&mut WlObjectStorage>) {
        let user_data_ptr = self.proxy().get_user_data().cast::<WlDispatchData<T>>();

        // Safety: the `WlObject` always has valid user data being set
        let user_data = unsafe { user_data_ptr.as_mut().unwrap_unchecked() };

        user_data.storage = Some(NonNull::new(&raw mut *storage).unwrap().cast());
    }

    pub fn proxy(&self) -> &WlProxy {
        &self.proxy
    }

    pub(crate) fn upcast(self) -> WlDynObject {
        // NOTE(hack3rmann): we can use `MaybeUninit` to
        // move out of `Self` which implements the `Drop` trait
        let mut this = MaybeUninit::new(self);

        WlDynObject {
            // Safety: here we moving out of `WlObject` without the destructor being called
            proxy: unsafe {
                this.as_mut_ptr()
                    .wrapping_byte_add(offset_of!(Self, proxy))
                    .cast::<WlProxy>()
                    .read()
            },
            type_info: TypeInfo::of::<T>(),
        }
    }
}

impl<T> Drop for WlObject<T> {
    fn drop(&mut self) {
        let user_data = self.proxy.get_user_data();

        // # Safety
        //
        // - `user_data` points to a valid instance of `WlDispatchData<T>`
        // - drop called once on a valid instance
        unsafe { drop(Box::from_raw(user_data.cast::<WlDispatchData<T>>())) };
    }
}

impl<T> Deref for WlObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let user_data = self.proxy.get_user_data();

        // Safety: `user_data` points to a valid instance of `WlDispatchData<T>`
        unsafe {
            &user_data
                .cast::<WlDispatchData<T>>()
                .as_ref()
                .unwrap_unchecked()
                .data
        }
    }
}

impl<T> DerefMut for WlObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let user_data = self.proxy.get_user_data();

        // Safety: `user_data` points to a valid instance of `WlDispatchData<T>`
        unsafe {
            &mut user_data
                .cast::<WlDispatchData<T>>()
                .as_mut()
                .unwrap_unchecked()
                .data
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for WlObject<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(any::type_name::<Self>())
            .field("proxy", &self.proxy)
            .field("data", self.deref())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::{compositor::WlCompositor, output::WlOutput, zwlr_layer_shell_v1::WlrLayerShellV1};
    use crate::{
        init::connect_wayland_socket,
        interface::{
            LayerSurfaceSetAnchorRequest, LayerSurfaceSetExclusiveZoneRequest,
            LayerSurfaceSetKeyboardInteractivityRequest, LayerSurfaceSetMarginRequest,
            LayerSurfaceSetSizeRequest, WlCompositorCreateRegion, WlCompositorCreateSurface,
            WlRegionDestroyRequest, WlShmCreatePoolRequest, WlShmFormat,
            WlShmPoolCreateBufferRequest, WlSurfaceAttachRequest, WlSurfaceCommitRequest,
            WlSurfaceDamageRequest, WlSurfaceSetBufferScaleRequest, WlSurfaceSetInputRegionRequest,
            WpViewporterGetViewportRequest, ZwlrLayerShellGetLayerSurfaceRequest,
            ZwlrLayerShellV1Layer,
            zwlr_layer_surface_v1::wl_enum::{Anchor, KeyboardInteractivity},
        },
        sys::{
            ObjectType,
            display::WlDisplay,
            object::{registry::WlRegistry, shm::WlShm, viewporter::WpViewporter},
            wire::SmallVecMessageBuffer,
        },
    };
    use rustix::{
        fs::Mode,
        mm::{MapFlags, ProtFlags},
        shm::OFlags,
    };
    use std::{
        mem,
        os::fd::{AsFd as _, OwnedFd},
        pin::pin,
        ptr, slice, thread,
        time::Duration,
    };

    unsafe fn connect_display() -> WlDisplay {
        let wayland_sock = unsafe { connect_wayland_socket().unwrap() };
        WlDisplay::connect_to_fd(wayland_sock).unwrap()
    }

    #[test]
    fn just_connect_displat() {
        _ = unsafe { connect_display() };
    }

    #[test]
    fn get_registry() {
        let mut buf = SmallVecMessageBuffer::<8>::new();

        // Safety: called once on the start of the program
        let display = unsafe { connect_display() };
        let mut storage = pin!(display.create_storage());
        let registry = display.create_registry(&mut buf, storage.as_mut());

        display.sync_all(storage.as_mut());

        assert!(
            storage
                .object(registry)
                .interfaces
                .contains_key(&ObjectType::Compositor)
        );
    }

    #[test]
    fn create_surface() {
        let mut buf = SmallVecMessageBuffer::<8>::new();

        // Safety: called once on the start of the program
        let display = unsafe { connect_display() };
        let mut storage = pin!(display.create_storage());
        let registry = display.create_registry(&mut buf, storage.as_mut());

        display.sync_all(storage.as_mut());

        let compositor =
            WlRegistry::bind_default::<WlCompositor>(&mut buf, storage.as_mut(), registry).unwrap();

        let surface =
            compositor.create_object(&mut buf, storage.as_mut(), WlCompositorCreateSurface);

        assert_eq!(
            storage.object(surface).proxy().interface_name(),
            "wl_surface",
        );
    }

    #[test]
    fn bind_wlr_shell() {
        let mut buf = SmallVecMessageBuffer::<8>::new();

        // Safety: called once on the start of the program
        let display = unsafe { connect_display() };
        let mut storage = pin!(display.create_storage());
        let registry = display.create_registry(&mut buf, storage.as_mut());

        display.sync_all(storage.as_mut());

        let _layer_shell =
            WlRegistry::bind_default::<WlrLayerShellV1>(&mut buf, storage.as_mut(), registry)
                .unwrap();

        display.sync_all(storage.as_mut());
    }

    fn open_shm() -> Result<(OwnedFd, String), rustix::io::Errno> {
        for i in 0.. {
            let wl_shm_path = format!("/wl_shm#{i}");

            match rustix::shm::open(
                &wl_shm_path,
                OFlags::EXCL | OFlags::RDWR | OFlags::CREATE | OFlags::TRUNC,
                Mode::RUSR | Mode::WUSR,
            ) {
                Ok(fd) => return Ok((fd, wl_shm_path)),
                Err(rustix::io::Errno::EXIST) => continue,
                Err(error) => return Err(error),
            };
        }

        unreachable!();
    }

    #[test]
    fn white_rect() {
        let mut buf = SmallVecMessageBuffer::<8>::new();

        // Safety: called once on the start of the program
        let display = unsafe { connect_display() };
        let mut storage = pin!(display.create_storage());
        let registry = display.create_registry(&mut buf, storage.as_mut());

        display.sync_all(storage.as_mut());

        let shm = WlRegistry::bind_default::<WlShm>(&mut buf, storage.as_mut(), registry).unwrap();

        let viewporter =
            WlRegistry::bind_default::<WpViewporter>(&mut buf, storage.as_mut(), registry).unwrap();

        let compositor =
            WlRegistry::bind_default::<WlCompositor>(&mut buf, storage.as_mut(), registry).unwrap();

        let surface =
            compositor.create_object(&mut buf, storage.as_mut(), WlCompositorCreateSurface);

        let _viewport = viewporter.create_object(
            &mut buf,
            storage.as_mut(),
            WpViewporterGetViewportRequest { surface },
        );

        let region = compositor.create_object(&mut buf, storage.as_mut(), WlCompositorCreateRegion);

        surface.request(
            &mut buf,
            &storage,
            WlSurfaceSetInputRegionRequest {
                region: Some(region),
            },
        );

        region.request(&mut buf, &storage, WlRegionDestroyRequest);

        let output =
            WlRegistry::bind_default::<WlOutput>(&mut buf, storage.as_mut(), registry).unwrap();

        let layer_shell =
            WlRegistry::bind_default::<WlrLayerShellV1>(&mut buf, storage.as_mut(), registry)
                .unwrap();

        let layer_surface = layer_shell.create_object(
            &mut buf,
            storage.as_mut(),
            ZwlrLayerShellGetLayerSurfaceRequest {
                surface,
                output: Some(output),
                layer: ZwlrLayerShellV1Layer::Background,
                namespace: c"wallpaper-engine",
            },
        );

        layer_surface.request(
            &mut buf,
            &storage,
            LayerSurfaceSetAnchorRequest {
                anchor: Anchor::all(),
            },
        );

        layer_surface.request(
            &mut buf,
            &storage,
            LayerSurfaceSetExclusiveZoneRequest { zone: -1 },
        );

        layer_surface.request(&mut buf, &storage, LayerSurfaceSetMarginRequest::zero());

        layer_surface.request(
            &mut buf,
            &storage,
            LayerSurfaceSetKeyboardInteractivityRequest {
                keyboard_interactivity: KeyboardInteractivity::None,
            },
        );

        layer_surface.request(
            &mut buf,
            &storage,
            LayerSurfaceSetSizeRequest {
                width: BUFFER_WIDTH_PIXELS as u32,
                height: BUFFER_HEIGHT_PIXELS as u32,
            },
        );

        surface.request(&mut buf, &storage, WlSurfaceCommitRequest);

        display.sync_all(storage.as_mut());

        let (shm_fd, shm_path) = open_shm().unwrap();

        const BUFFER_WIDTH_PIXELS: usize = 2520;
        const BUFFER_HEIGHT_PIXELS: usize = 1680;
        const PIXEL_SIZE_BYTES: usize = mem::size_of::<u32>();
        const BUFFER_SIZE_PIXELS: usize = BUFFER_WIDTH_PIXELS * BUFFER_HEIGHT_PIXELS;
        const BUFFER_SIZE_BYTES: usize = BUFFER_SIZE_PIXELS * PIXEL_SIZE_BYTES;

        rustix::fs::ftruncate(&shm_fd, BUFFER_SIZE_BYTES as u64).unwrap();

        let shm_ptr = unsafe {
            rustix::mm::mmap(
                ptr::null_mut(),
                BUFFER_SIZE_BYTES,
                ProtFlags::READ | ProtFlags::WRITE,
                MapFlags::SHARED,
                &shm_fd,
                0,
            )
            .unwrap()
            .cast::<u32>()
        };

        rustix::shm::unlink(&shm_path).unwrap();

        assert!(!shm_ptr.is_null());
        assert!(shm_ptr.is_aligned());

        unsafe { shm_ptr.write_bytes(0xFF, BUFFER_SIZE_PIXELS) };

        let _buffer =
            unsafe { slice::from_raw_parts_mut(shm_ptr.cast::<u32>(), BUFFER_SIZE_PIXELS) };

        let shm_pool = shm.create_object(
            &mut buf,
            storage.as_mut(),
            WlShmCreatePoolRequest {
                fd: shm_fd.as_fd(),
                size: BUFFER_SIZE_BYTES as i32,
            },
        );

        let buffer = shm_pool.create_object(
            &mut buf,
            storage.as_mut(),
            WlShmPoolCreateBufferRequest {
                offset: 0,
                width: BUFFER_WIDTH_PIXELS as i32,
                height: BUFFER_HEIGHT_PIXELS as i32,
                stride: (BUFFER_WIDTH_PIXELS * PIXEL_SIZE_BYTES) as i32,
                format: WlShmFormat::Xrgb8888,
            },
        );

        surface.request(
            &mut buf,
            &storage,
            WlSurfaceSetBufferScaleRequest { scale: 1 },
        );

        surface.request(
            &mut buf,
            &storage,
            WlSurfaceAttachRequest {
                buffer: Some(buffer),
                x: 0,
                y: 0,
            },
        );

        surface.request(
            &mut buf,
            &storage,
            WlSurfaceDamageRequest {
                x: 0,
                y: 0,
                width: BUFFER_WIDTH_PIXELS as i32,
                height: BUFFER_HEIGHT_PIXELS as i32,
            },
        );

        surface.request(&mut buf, &storage, WlSurfaceCommitRequest);

        display.sync_all(storage.as_mut());

        thread::sleep(Duration::from_millis(200));
    }
}
