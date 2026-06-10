use crate::status::{clear_progress_line, stderr_is_tty, transient_status, write_progress_line};
use anstyle_progress::TermProgress;
use std::{
    cmp, env,
    fmt::{self, Display},
    time::{Duration, Instant},
};
use unicode_width::UnicodeWidthChar;

pub struct Progress {
    state: Option<State>,
}

impl Progress {
    pub fn new(name: &str) -> Self {
        if !progress_enabled() {
            return Self { state: None };
        }

        let width = stderr_width().progress_max_width();
        Self {
            state: width.map(|max_width| State {
                format: Format {
                    max_width,
                    max_print: 50,
                    term_integration: TerminalIntegration::new(),
                    unicode: supports_unicode(),
                },
                name: name.to_string(),
                done: false,
                throttle: Throttle::new(),
                last_line: None,
            }),
        }
    }

    pub fn tick(&mut self, cur: usize, max: usize, msg: &str) {
        let Some(state) = &mut self.state else {
            return;
        };

        if !state.throttle.allowed() {
            return;
        }

        if max > 0 && cur == max {
            state.done = true;
        }

        if let Some(pbar) = state.format.progress(cur, max) {
            state.print(pbar, msg);
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
        if let Some(state) = &mut self.state {
            if state.format.term_integration.enabled {
                write_progress_line(&StatusValue::Remove.to_string());
            }
            if state.last_line.is_some() {
                clear_progress_line();
                state.last_line = None;
            }
        }
    }
}

fn progress_enabled() -> bool {
    if !stderr_is_tty() {
        return false;
    }

    !env::var_os("CI").is_some()
}

enum TtyWidth {
    NoTty,
    Known(usize),
}

impl TtyWidth {
    fn progress_max_width(self) -> Option<usize> {
        match self {
            TtyWidth::NoTty => None,
            TtyWidth::Known(width) => Some(width),
        }
    }
}

const DEFAULT_PROGRESS_WIDTH: usize = 80;

fn stderr_width() -> TtyWidth {
    if !stderr_is_tty() {
        return TtyWidth::NoTty;
    }

    #[cfg(unix)]
    {
        unsafe {
            let mut winsize: libc::winsize = std::mem::zeroed();
            if libc::ioctl(libc::STDERR_FILENO, libc::TIOCGWINSZ, &mut winsize) >= 0
                && winsize.ws_col > 0
            {
                return TtyWidth::Known(winsize.ws_col as usize);
            }
        }
    }

    if let Ok(columns) = env::var("COLUMNS")
        && let Ok(width) = columns.parse::<usize>()
        && width > 0
    {
        return TtyWidth::Known(width);
    }

    TtyWidth::Known(DEFAULT_PROGRESS_WIDTH)
}

fn supports_unicode() -> bool {
    true
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

        let (mut line, report) = match progress {
            ProgressOutput::TextAndReport(prefix, report) => (prefix, Some(report)),
        };

        if self.format.max_width < 15 {
            if let Some(report) = report {
                write_progress_line(&report.to_string());
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

struct Format {
    max_width: usize,
    max_print: usize,
    term_integration: TerminalIntegration,
    unicode: bool,
}

impl Format {
    fn progress(&self, cur: usize, max: usize) -> Option<ProgressOutput> {
        assert!(cur <= max);

        let pct = if max == 0 {
            0.0
        } else {
            (cur as f64) / (max as f64)
        };
        let pct = if !pct.is_finite() { 0.0 } else { pct };

        let stats = format!(" {cur}/{max}");
        let report = {
            let pct = (pct * 100.0) as u8;
            let pct = pct.clamp(0, 100);
            self.term_integration.value(pct)
        };

        let extra_len = stats.len() + 2 + 15;
        let display_width = self.width().checked_sub(extra_len)?;

        let mut string = String::with_capacity(self.max_width);
        string.push('[');
        let hashes = (display_width as f64 * pct) as usize;

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

        for _ in 0..display_width.saturating_sub(hashes) {
            string.push(' ');
        }
        string.push(']');
        string.push_str(&stats);

        Some(ProgressOutput::TextAndReport(string, report))
    }

    fn render(&self, string: &mut String, msg: &str) {
        if msg.is_empty() {
            return;
        }

        string.push_str(": ");
        let mut avail_msg_len = self.max_width.saturating_sub(string.len() + 15);
        let mut ellipsis_pos = 0;

        let (ellipsis, ellipsis_width) = if self.unicode { ("…", 1) } else { ("...", 3) };

        if avail_msg_len <= ellipsis_width {
            return;
        }

        for c in msg.chars() {
            let display_width = c.width().unwrap_or(0);
            if avail_msg_len >= display_width {
                avail_msg_len -= display_width;
                string.push(c);
                if avail_msg_len >= ellipsis_width {
                    ellipsis_pos = string.len();
                }
            } else {
                string.truncate(ellipsis_pos);
                string.push_str(ellipsis);
                break;
            }
        }
    }

    fn width(&self) -> usize {
        cmp::min(self.max_width, self.max_print)
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
    enabled: bool,
}

impl TerminalIntegration {
    fn new() -> Self {
        Self {
            enabled: anstyle_progress::supports_term_progress(stderr_is_tty()),
        }
    }

    fn value(&self, percent: u8) -> StatusValue {
        if self.enabled {
            StatusValue::Value(percent)
        } else {
            StatusValue::None
        }
    }
}

enum ProgressOutput {
    TextAndReport(String, StatusValue),
}

enum StatusValue {
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
