use crate::event::EventEmitter;
use std::{panic, thread::JoinHandle};

pub struct TaskPool<T> {
    pub handles: Vec<JoinHandle<()>>,
    pub emitter: EventEmitter<T>,
}

impl<T> TaskPool<T> {
    pub fn new(emitter: EventEmitter<T>) -> Self {
        Self {
            handles: vec![],
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

                if let Err(payload) = handle.join() {
                    panic::resume_unwind(payload);
                }
            }
        }

        n_finished
    }

    pub fn spawn<F>(&mut self, f: F)
    where
        F: FnOnce(EventEmitter<T>) + Send + 'static,
        T: Send + 'static,
    {
        self.erase_finished();

        let emitter = self.emitter.clone();
        let handle = std::thread::spawn(move || f(emitter));

        self.handles.push(handle);
    }
}
