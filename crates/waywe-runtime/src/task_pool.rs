use crate::event::EventEmitter;
use smallvec::{SmallVec, smallvec};
use std::{
    any::Any,
    fmt::{self, Display},
    thread::{self, JoinHandle},
};
use tracing::error;

struct PrettyPanicPayload<'s>(pub &'s (dyn Any + Send + 'static));

impl Display for PrettyPanicPayload<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = if let Some(s) = self.0.downcast_ref::<&str>() {
            s
        } else if let Some(s) = self.0.downcast_ref::<&String>() {
            s
        } else {
            "unknown panic payload"
        };

        f.write_str(s)
    }
}

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

    pub fn erase_finished(&mut self) -> usize {
        let mut n_finished = 0;
        let mut i = 0;

        while i < self.handles.len() {
            while i < self.handles.len() && self.handles[i].is_finished() {
                let handle = self.handles.swap_remove(i);

                if let Err(panic_payload) = handle.join() {
                    error!("task failed: {}", PrettyPanicPayload(&panic_payload));
                }

                n_finished += 1;
            }

            i += 1;
        }

        n_finished
    }

    pub fn spawn(&mut self, f: impl FnOnce(EventEmitter) + Send + 'static) {
        self.erase_finished();

        let emitter = self.emitter.clone();
        let handle = thread::spawn(move || f(emitter));

        self.handles.push(handle);
    }
}
