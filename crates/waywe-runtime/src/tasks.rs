use crate::event::EventEmitter;
use display_error_chain::ErrorChainExt;
use std::collections::HashMap;
use tokio::task::{AbortHandle, Id as TaskId, JoinError, JoinSet};
use tracing::error;

type OnErrorFn = Box<dyn FnOnce(JoinError) + Send>;

pub struct Tasks {
    pub set: JoinSet<()>,
    pub emitter: EventEmitter,
    pub on_error: HashMap<TaskId, OnErrorFn>,
}

impl Tasks {
    pub fn new(emitter: EventEmitter) -> Self {
        Self {
            set: JoinSet::new(),
            emitter,
            on_error: HashMap::new(),
        }
    }

    pub fn erase_finished(&mut self) {
        while let Some(result) = self.set.try_join_next_with_id() {
            match result {
                Ok((id, ())) => {
                    self.on_error.remove(&id);
                }
                Err(error) => {
                    error!("task failed: {}", error.chain());

                    let Some(on_error) = self.on_error.remove(&error.id()) else {
                        continue;
                    };
                    on_error(error);
                }
            }
        }
    }

    pub fn spawn<F, R>(&mut self, f: F) -> TaskOperations<'_>
    where
        F: FnOnce(EventEmitter) -> R + Send + 'static,
        R: Future<Output = ()> + Send + 'static,
    {
        self.erase_finished();

        let emitter = self.emitter.clone();
        let handle = self.set.spawn(async move {
            f(emitter).await;
        });

        TaskOperations {
            handle,
            tasks: self,
        }
    }
}

pub struct TaskOperations<'t> {
    pub handle: AbortHandle,
    tasks: &'t mut Tasks,
}

impl TaskOperations<'_> {
    pub fn on_error(&mut self, on: impl FnOnce(JoinError) + Send + 'static) -> &mut Self {
        self.tasks.on_error.insert(self.handle.id(), Box::new(on));
        self
    }
}
