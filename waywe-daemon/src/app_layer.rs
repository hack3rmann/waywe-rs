use crate::{
    event::{DynEventHandler, Event, EventHandler},
    event_loop::{FrameError, FrameInfo},
    runtime::Runtime,
};
use reusable_box::{ReusableBox, ReusedBoxFuture};
use static_assertions::assert_impl_all;
use std::{any::Any, ptr::NonNull};

pub trait AppLayer: Any + Send + Sync {
    fn populate_handler(&mut self, handler: &mut EventHandler<Self>)
    where
        Self: Sized;

    fn frame(
        &mut self,
        runtime: &mut Runtime,
    ) -> impl Future<Output = Result<FrameInfo, FrameError>> + Send;

    fn exit(&mut self, _runtime: &mut Runtime) -> impl Future<Output = ()> + Send {
        async {}
    }
}

type FrameFn = for<'f> unsafe fn(
    layer: NonNull<()>,
    runtime: &'f mut Runtime,
    future: &'f mut ReusableBox,
) -> ReusedBoxFuture<'f, Result<FrameInfo, FrameError>>;
assert_impl_all!(FrameFn: Copy);

type ExitFn = for<'f> unsafe fn(
    layer: NonNull<()>,
    runtime: &'f mut Runtime,
    future: &'f mut ReusableBox,
) -> ReusedBoxFuture<'f, ()>;
assert_impl_all!(ExitFn: Copy);

unsafe fn frame<'f, L: AppLayer>(
    layer: NonNull<()>,
    runtime: &'f mut Runtime,
    future: &'f mut ReusableBox,
) -> ReusedBoxFuture<'f, Result<FrameInfo, FrameError>> {
    let layer = unsafe { layer.cast::<L>().as_mut() };
    future.store_future(layer.frame(runtime))
}

unsafe fn exit<'f, L: AppLayer>(
    layer: NonNull<()>,
    runtime: &'f mut Runtime,
    future: &'f mut ReusableBox,
) -> ReusedBoxFuture<'f, ()> {
    let layer = unsafe { layer.cast::<L>().as_mut() };
    future.store_future(layer.exit(runtime))
}

pub struct DynAppLayer {
    frame: FrameFn,
    exit: ExitFn,
    layer: Box<dyn Any>,
    futures: ReusableBox,
    handler: DynEventHandler,
}

impl DynAppLayer {
    pub fn new<L: AppLayer>(mut layer: L) -> Self {
        let mut handler = EventHandler::<L>::default();
        layer.populate_handler(&mut handler);

        Self {
            frame: frame::<L>,
            exit: exit::<L>,
            layer: Box::new(layer),
            futures: ReusableBox::new(),
            handler: handler.into(),
        }
    }

    pub async fn handle_event(&mut self, runtime: &mut Runtime, event: &mut Event) {
        let layer_ptr = unsafe { NonNull::new_unchecked((&raw mut *self.layer).cast::<()>()) };
        unsafe { self.handler.execute_all(layer_ptr, runtime, event) }.await;
    }

    pub async fn frame(&mut self, runtime: &mut Runtime) -> Result<FrameInfo, FrameError> {
        let layer_ptr = unsafe { NonNull::new_unchecked((&raw mut *self.layer).cast::<()>()) };
        let future = unsafe { (self.frame)(layer_ptr, runtime, &mut self.futures) };
        future.await
    }

    pub async fn exit(&mut self, runtime: &mut Runtime) {
        let layer_ptr = unsafe { NonNull::new_unchecked((&raw mut *self.layer).cast::<()>()) };
        let future = unsafe { (self.exit)(layer_ptr, runtime, &mut self.futures) };
        future.await;
    }
}
