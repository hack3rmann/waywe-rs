use crate::event::EventEmitter;
use std::thread::JoinHandle;

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

    pub fn spawn<F>(&mut self, f: F)
    where
        F: FnOnce(EventEmitter<T>) + Send + 'static,
        T: Send + 'static,
    {
        let emitter = self.emitter.clone();
        let handle = std::thread::spawn(move || f(emitter));
        self.handles.push(handle);
    }
}
