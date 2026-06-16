use crate::event::{EventEmitter, IntoEvent};
use display_error_chain::ErrorChainExt;
use smallvec::{SmallVec, smallvec};
use tokio::task::JoinHandle;
use tracing::error;

pub struct TaskPool {
    pub handles: SmallVec<[JoinHandle<()>; 1]>,
    pub emitter: EventEmitter,
}

impl TaskPool {
    pub fn new(emitter: EventEmitter) -> Self {
        Self {
            handles: smallvec![],
            emitter,
        }
    }

    pub async fn erase_finished(&mut self) -> usize {
        let mut n_finished = 0;
        let mut i = 0;

        while i < self.handles.len() {
            while i < self.handles.len() && self.handles[i].is_finished() {
                let handle = self.handles.swap_remove(i);

                if let Err(err) = handle.await {
                    error!("task failed: {}", err.chain());
                }

                n_finished += 1;
            }

            i += 1;
        }

        n_finished
    }

    pub async fn spawn<F, R>(&mut self, f: F)
    where
        F: FnOnce(EventEmitter) -> R + Send + 'static,
        R: Future<Output = ()> + Send + 'static,
    {
        self.erase_finished().await;

        let emitter = self.emitter.clone();
        let handle = tokio::spawn(async move {
            f(emitter).await;
        });

        self.handles.push(handle);
    }

    pub async fn spawn_event<F, R, E>(&mut self, f: F)
    where
        F: FnOnce() -> R + Send + 'static,
        R: Future<Output = E> + Send + 'static,
        E: IntoEvent,
    {
        self.spawn(async move |mut emitter| {
            let event = f().await;
            emitter.emit(event).expect("failed to send event");
        })
        .await;
    }
}
