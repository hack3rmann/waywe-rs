pub mod almost;
pub mod app;
pub mod event_loop;
pub mod image_pipeline;
pub mod runtime;
pub mod video_pipeline;
pub mod wallpaper;

use app::VideoApp;
use event_loop::EventLoop;

fn main() {
    EventLoop::<VideoApp>::default().run();
}
