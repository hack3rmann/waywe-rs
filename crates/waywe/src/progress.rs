use crate::status::{clear_progress_line, stderr_is_tty, transient_status, write_progress_line};
use anstyle_progress::TermProgress;
use std::{
    env,
    fmt::{self, Display, Write},
    time::{Duration, Instant},
};
use unicode_width::UnicodeWidthChar;

pub struct Progress {
    state: Option<State>,
}

impl Progress {
    pub fn new(name: impl Into<String>) -> Self {
        if !stderr_is_tty() || env::var_os("CI").is_some() {
            return Self { state: None };
        }

        let state = stderr_width().map(|max_width| State {
            format: Format {
                max_width,
                max_print: 50,
                term_integration: TerminalIntegration::new(),
            },
            name: name.into(),
            done: false,
            throttle: Throttle::new(),
            last_line: None,
        });

        Self { state }
    }

    pub fn update(&mut self, cur: usize, max: usize, msg: &str) {
        let Some(state) = &mut self.state else {
            return;
        };

        if !state.throttle.allowed() {
            return;
        }

        if max > 0 && cur == max {
            state.done = true;
        }

        if let Some(output) = state.format.progress(cur, max) {
            state.print(output, msg);
        }
    }
}

impl Drop for Progress {
    fn drop(&mut self) {
        self.clear();
    }
}

impl Progress {
    pub fn clear(&mut self) {
        let Some(state) = &mut self.state else { return };

        if state.format.term_integration.is_enabled {
            write_progress_line(&StatusValue::Remove);
        }

        if state.last_line.is_some() {
            clear_progress_line();
            state.last_line = None;
        }
    }
}

const DEFAULT_PROGRESS_WIDTH: usize = 80;

#[cfg(unix)]
fn stderr_width_unix() -> Option<usize> {
    use std::mem::MaybeUninit;

    let mut winsize: MaybeUninit<libc::winsize> = MaybeUninit::uninit();

    if unsafe { libc::ioctl(libc::STDERR_FILENO, libc::TIOCGWINSZ, winsize.as_mut_ptr()) } < 0 {
        return None;
    }

    // Safety: ioctl returned success, therefore winsize is initialized
    let winsize = unsafe { winsize.assume_init() };

    (winsize.ws_col > 0).then_some(winsize.ws_col as usize)
}

fn stderr_width() -> Option<usize> {
    if !stderr_is_tty() {
        return None;
    }

    #[cfg(unix)]
    if let Some(size) = stderr_width_unix() {
        return Some(size);
    }

    if let Ok(columns) = env::var("COLUMNS")
        && let Ok(width) = columns.parse::<usize>()
        && width > 0
    {
        return Some(width);
    }

    Some(DEFAULT_PROGRESS_WIDTH)
}

struct State {
    format: Format,
    name: String,
    done: bool,
    throttle: Throttle,
    last_line: Option<String>,
}

impl State {
    fn print(&mut self, progress: ProgressOutput, msg: &str) {
        self.throttle.update();

        let mut line = progress.text;
        let report = Some(progress.report);

        if self.format.max_width < 15 {
            if let Some(report) = report {
                write_progress_line(&report);
            }

            return;
        }

        self.format.render(&mut line, msg);

        while line.len() < self.format.max_width.saturating_sub(15) {
            line.push(' ');
        }

        let full_line = if let Some(report) = report {
            format!("{line}{report}")
        } else {
            line.clone()
        };

        if self.last_line.as_ref() != Some(&line) {
            transient_status(&self.name);
            write_progress_line(&full_line);
            self.last_line = Some(line);
        }
    }
}

const fn calculate_done_fraction(cur: usize, max: usize) -> f32 {
    let done_fraction = if max == 0 {
        0.0
    } else {
        (cur as f32) / (max as f32)
    };

    if !done_fraction.is_finite() {
        0.0
    } else {
        done_fraction
    }
}

struct Format {
    max_width: usize,
    max_print: usize,
    term_integration: TerminalIntegration,
}

impl Format {
    fn progress(&self, cur: usize, max: usize) -> Option<ProgressOutput> {
        assert!(cur <= max);

        let done_fraction = calculate_done_fraction(cur, max);

        let percentage = (done_fraction * 100.0).clamp(0.0, 100.0) as u8;
        let report = self.term_integration.value(percentage);

        let stats = format!(" {cur}/{max}");
        let extra_len = stats.len() + 2 + 15;
        let display_width = self.width().checked_sub(extra_len)?;

        let mut string = String::with_capacity(self.max_width);
        string.push('[');
        let hashes = (display_width as f32 * done_fraction) as usize;

        if hashes > 0 {
            for _ in 0..hashes.saturating_sub(1) {
                string.push('=');
            }

            if cur == max {
                string.push('=');
            } else {
                string.push('>');
            }
        }

        let spaces = (0..display_width.saturating_sub(hashes)).map(|_| ' ');
        string.extend(spaces);

        write!(&mut string, "]{stats}").unwrap();

        Some(ProgressOutput {
            text: string,
            report,
        })
    }

    fn render(&self, result: &mut String, msg: &str) {
        if msg.is_empty() {
            return;
        }

        result.push_str(": ");

        let mut avail_msg_len = self.max_width.saturating_sub(result.len() + 15);
        let mut ellipsis_pos = 0;

        const ELLIPSIS: &str = "…";

        if avail_msg_len <= ELLIPSIS.len() {
            return;
        }

        for c in msg.chars() {
            let display_width = c.width().unwrap_or(0);

            if avail_msg_len < display_width {
                result.truncate(ellipsis_pos);
                result.push_str(ELLIPSIS);
                break;
            }

            avail_msg_len -= display_width;
            result.push(c);

            if avail_msg_len >= ELLIPSIS.len() {
                ellipsis_pos = result.len();
            }
        }
    }

    fn width(&self) -> usize {
        self.max_width.min(self.max_print)
    }
}

struct Throttle {
    first: bool,
    last_update: Instant,
}

impl Throttle {
    fn new() -> Self {
        Self {
            first: true,
            last_update: Instant::now(),
        }
    }

    fn allowed(&mut self) -> bool {
        if self.first {
            self.update();
            return true;
        }

        if self.last_update.elapsed() < Duration::from_millis(100) {
            return false;
        }

        self.update();
        true
    }

    fn update(&mut self) {
        self.first = false;
        self.last_update = Instant::now();
    }
}

struct TerminalIntegration {
    is_enabled: bool,
}

impl TerminalIntegration {
    fn new() -> Self {
        Self {
            is_enabled: anstyle_progress::supports_term_progress(stderr_is_tty()),
        }
    }

    fn value(&self, percent: u8) -> StatusValue {
        if self.is_enabled {
            StatusValue::Value(percent)
        } else {
            StatusValue::None
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
struct ProgressOutput {
    pub text: String,
    pub report: StatusValue,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum StatusValue {
    #[default]
    None,
    Remove,
    Value(u8),
}

impl Display for StatusValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => TermProgress::none().fmt(f),
            Self::Remove => TermProgress::remove().fmt(f),
            Self::Value(v) => TermProgress::start().percent(*v).fmt(f),
        }
    }
}
