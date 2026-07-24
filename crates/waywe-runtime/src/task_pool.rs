use crate::event::{EventEmitter, IntoEvent};
use display_error_chain::ErrorChainExt;
use slab::Slab;
use smallvec::SmallVec;
use tokio::task::JoinHandle;
use tracing::error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskIndex(usize);

pub struct TaskPool {
    pub handles: Slab<JoinHandle<()>>,
    pub emitter: EventEmitter,
}

impl TaskPool {
    pub fn new(emitter: EventEmitter) -> Self {
        Self {
            handles: Slab::new(),
            emitter,
        }
    }

    pub async fn erase_finished(&mut self) -> usize {
        let mut n_finished = 0;
        let mut finished = SmallVec::<[usize; 8]>::new_const();

        loop {
            for (i, handle) in &mut self.handles {
                if handle.is_finished() {
                    finished.push(i);
                }
            }

            if finished.is_empty() {
                break;
            }

            for i in finished.drain(..) {
                let Some(handle) = self.handles.try_remove(i) else {
                    continue;
                };

                if let Err(error) = handle.await {
                    error!("task failed: {}", error.chain());
                }

                n_finished += 1;
            }
        }

        n_finished
    }

    pub async fn spawn<F, R>(&mut self, f: F) -> TaskIndex
    where
        F: FnOnce(EventEmitter) -> R + Send + 'static,
        R: Future<Output = ()> + Send + 'static,
    {
        self.erase_finished().await;

        let emitter = self.emitter.clone();
        let handle = tokio::spawn(async move {
            f(emitter).await;
        });

        TaskIndex(self.handles.insert(handle))
    }

    pub async fn spawn_event<F, R, E>(&mut self, f: F) -> TaskIndex
    where
        F: FnOnce() -> R + Send + 'static,
        R: Future<Output = E> + Send + 'static,
        E: IntoEvent,
    {
        self.spawn(async move |mut emitter| {
            let event = f().await;
            emitter.try_emit(event).expect("failed to send event");
        })
        .await
    }

    pub async fn terminate(&mut self, task_id: TaskIndex) {
        let Some(handle) = self.handles.try_remove(task_id.0) else {
            return;
        };

        // Try to shutdown the task gracefully
        if handle.is_finished()
            && let Err(error) = handle.await
        {
            error!("task failed: {}", error.chain());
        }
    }
}
