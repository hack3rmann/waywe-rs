use std::time::{Duration, Instant};

#[allow(unused)]
pub struct Timer {
    pub init_time: Instant,
    pub event_loop_start_time: Option<Instant>,
    pub wallpaper_start_time: Option<Instant>,
    pub current_frame_start_time: Option<Instant>,
    pub duration_since_last_frame: Duration,
    pub time_borrow: Duration,
    pub frame_index: usize,
}

impl Timer {
    pub fn is_first_frame(&self) -> bool {
        self.frame_index == 0
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            init_time: Instant::now(),
            event_loop_start_time: None,
            wallpaper_start_time: None,
            current_frame_start_time: None,
            duration_since_last_frame: Duration::default(),
            time_borrow: Duration::default(),
            frame_index: usize::MAX,
        }
    }
}

impl Timer {
    pub fn mark_event_loop_start_time(&mut self) {
        _ = self.event_loop_start_time.insert(Instant::now());
    }

    pub fn mark_wallpaper_start_time(&mut self) {
        _ = self.wallpaper_start_time.insert(Instant::now());
    }

    pub fn mark_frame_start(&mut self) {
        let now = Instant::now();
        let frame_start = self.current_frame_start_time.get_or_insert(now);

        self.duration_since_last_frame = now.duration_since(*frame_start);
        *frame_start = now;

        self.frame_index = self.frame_index.wrapping_add(1);
    }

    pub fn last_frame_duration(&self) -> Duration {
        self.duration_since_last_frame
    }

    pub fn current_frame_duration(&self) -> Duration {
        let Some(start_time) = self.current_frame_start_time else {
            return Duration::default();
        };

        Instant::now().duration_since(start_time)
    }
}
