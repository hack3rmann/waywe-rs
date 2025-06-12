use std::sync::Once;

pub struct Video {}

impl Default for Video {
    fn default() -> Self {
        static ONCE: Once = Once::new();
        ONCE.call_once(video::init);
        Self {}
    }
}
