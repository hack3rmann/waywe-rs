use std::{
    thread,
    time::{Duration, Instant},
};

#[allow(unused)]
pub struct Timer {
    pub init_time: Instant,
    pub event_loop_start_time: Option<Instant>,
    pub wallpaper_start_time: Option<Instant>,
    pub current_frame_start_time: Option<Instant>,
    pub duration_since_last_frame: Duration,
    pub time_borrow: Duration,
    pub frame_index: usize,
    pub block_start: Option<Instant>,
    pub block_duration: Duration,
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
            duration_since_last_frame: Duration::ZERO,
            time_borrow: Duration::ZERO,
            frame_index: usize::MAX,
            block_start: None,
            block_duration: Duration::ZERO,
        }
    }
}

impl Timer {
    pub fn mark_event_loop_start_time(&mut self) {
        self.event_loop_start_time = Some(Instant::now());
    }

    pub fn mark_wallpaper_start_time(&mut self) {
        self.frame_index = 0;
        self.wallpaper_start_time = Some(Instant::now());
        self.time_borrow = Duration::ZERO;
    }

    pub fn mark_frame_start(&mut self) {
        self.block_start = None;
        self.block_duration = Duration::ZERO;

        let now = Instant::now();
        let frame_start = self.current_frame_start_time.get_or_insert(now);

        self.duration_since_last_frame = now.duration_since(*frame_start);
        *frame_start = now;

        self.frame_index = self.frame_index.wrapping_add(1);
    }

    pub fn mark_block_start(&mut self) {
        self.block_start = Some(Instant::now());
    }

    pub fn mark_block_end(&mut self) -> Duration {
        let Some(start) = self.block_start else {
            return Duration::ZERO;
        };

        self.block_duration = start.elapsed();
        self.block_duration
    }

    pub fn last_frame_duration(&self) -> Duration {
        self.duration_since_last_frame
            .saturating_sub(self.block_duration)
    }

    pub fn current_frame_duration(&self) -> Duration {
        let Some(start_time) = self.current_frame_start_time else {
            return Duration::default();
        };

        start_time.elapsed().saturating_sub(self.block_duration)
    }

    pub fn sleep_enough(&mut self, target_frame_time: Duration) {
        let render_time = self.current_frame_duration();
        let sleep_time = target_frame_time.saturating_sub(render_time);

        // TODO(hack3rmann): skip a frame if `time_borrow >= target_frame_time`
        if !sleep_time.is_zero() {
            let unborrowed_time = sleep_time.saturating_sub(self.time_borrow);

            if !unborrowed_time.is_zero() {
                self.time_borrow = Duration::default();
                thread::sleep(unborrowed_time);
            } else {
                self.time_borrow -= sleep_time;
            }
        // ignore first frame lag
        } else if !self.is_first_frame() {
            self.time_borrow += render_time - target_frame_time;
        }
    }
}
