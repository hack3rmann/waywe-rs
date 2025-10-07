use crate::{
    Runtime,
    event::{DynEventHandler, Event, EventHandler},
    frame::{FrameError, FrameInfo},
};
use futures_util::Future;
use reusable_box::{ReusableBox, ReusedBoxFuture};
use std::{any::Any, ptr::NonNull};

pub trait App: Any + Send + Sync {
    fn populate_handler(&mut self, handler: &mut EventHandler<Self>);

    fn frame(
        &mut self,
        runtime: &mut Runtime,
    ) -> impl Future<Output = Result<FrameInfo, FrameError>> + Send;

    #[expect(unused_variables)]
    fn exit(&mut self, runtime: &mut Runtime) -> impl Future<Output = ()> + Send {
        async {}
    }
}

pub trait ReusableApp: Any + Send + Sync {
    fn frame<'f>(
        &'f mut self,
        runtime: &'f mut Runtime,
        futures: &'f mut ReusableBox,
    ) -> ReusedBoxFuture<'f, Result<FrameInfo, FrameError>>;

    fn exit<'f>(
        &'f mut self,
        runtime: &'f mut Runtime,
        futures: &'f mut ReusableBox,
    ) -> ReusedBoxFuture<'f, ()>;
}

impl<A: App> ReusableApp for A {
    fn frame<'f>(
        &'f mut self,
        runtime: &'f mut Runtime,
        futures: &'f mut ReusableBox,
    ) -> ReusedBoxFuture<'f, Result<FrameInfo, FrameError>> {
        futures.store_future(App::frame(self, runtime))
    }

    fn exit<'f>(
        &'f mut self,
        runtime: &'f mut Runtime,
        futures: &'f mut ReusableBox,
    ) -> ReusedBoxFuture<'f, ()> {
        futures.store_future(App::exit(self, runtime))
    }
}

pub struct DynApp {
    app: Box<dyn ReusableApp>,
    futures: ReusableBox,
    handler: DynEventHandler,
}

impl DynApp {
    pub fn new(mut app: impl App) -> Self {
        let mut handler = EventHandler::default();
        app.populate_handler(&mut handler);

        Self {
            app: Box::new(app),
            futures: ReusableBox::new(),
            handler: handler.into(),
        }
    }

    pub async fn handle_event(&mut self, runtime: &mut Runtime, event: &mut Event) {
        let layer_ptr = unsafe { NonNull::new_unchecked((&raw mut *self.app).cast::<()>()) };
        unsafe { self.handler.execute_all(layer_ptr, runtime, event) }.await;
    }

    pub async fn frame(&mut self, runtime: &mut Runtime) -> Result<FrameInfo, FrameError> {
        self.app.frame(runtime, &mut self.futures).await
    }

    pub async fn exit(&mut self, runtime: &mut Runtime) {
        self.app.exit(runtime, &mut self.futures).await;
    }
}
