use crate::event::EventEmitter;
use smallvec::{SmallVec, smallvec};
use std::thread::JoinHandle;
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

    pub fn erase_finished(&mut self) -> usize {
        let n_finished = 0;

        for i in 0..self.handles.len() {
            if i + n_finished >= self.handles.len() {
                break;
            }

            if self.handles[i].is_finished() {
                let handle = self.handles.swap_remove(i);

                if let Err(_panic_payload) = handle.join() {
                    error!("task failed");
                }
            }
        }

        n_finished
    }

    pub fn spawn(&mut self, f: impl FnOnce(EventEmitter) + Send + 'static) {
        self.erase_finished();

        let emitter = self.emitter.clone();
        let handle = std::thread::spawn(move || f(emitter));

        self.handles.push(handle);
    }
}
