use anstream::AutoStream;
use anstyle::{AnsiColor, Effects, Style};
use std::{
    cell::Cell,
    fmt::Display,
    io::{IsTerminal, Write},
    time::Duration,
};

const HEADER: Style = AnsiColor::BrightGreen.on_default().effects(Effects::BOLD);
pub(crate) const TRANSIENT: Style = AnsiColor::BrightCyan.on_default().effects(Effects::BOLD);

thread_local! {
    static NEEDS_CLEAR: Cell<bool> = const { Cell::new(false) };
}

pub fn set_needs_clear(needs_clear: bool) {
    NEEDS_CLEAR.with(|flag| flag.set(needs_clear));
}

pub fn is_cleared() -> bool {
    NEEDS_CLEAR.with(|flag| !flag.get())
}

fn erase_line_if_needed(stderr: &mut AutoStream<std::io::Stderr>) {
    NEEDS_CLEAR.with(|flag| {
        if flag.get() {
            let _ = stderr.write_all(b"\x1B[K");
            flag.set(false);
        }
    });
}

pub fn status(label: &str, message: impl Display) {
    let mut stderr = AutoStream::auto(std::io::stderr());
    erase_line_if_needed(&mut stderr);
    let _ = write!(stderr, "{HEADER}{label:>12}{HEADER:#} {message}\n");
}

pub fn transient_status(label: &str) {
    let mut stderr = AutoStream::auto(std::io::stderr());
    erase_line_if_needed(&mut stderr);
    let _ = write!(stderr, "{TRANSIENT}{label:>12}{TRANSIENT:#} ");
}

pub fn write_progress_line(line: &str) {
    let mut stderr = AutoStream::auto(std::io::stderr());
    let _ = write!(stderr, "{line}\r");
    set_needs_clear(true);
}

pub fn clear_progress_line() {
    if !is_cleared() {
        let mut stderr = AutoStream::auto(std::io::stderr());
        let _ = stderr.write_all(b"\x1B[K");
        set_needs_clear(false);
    }
}

pub fn stderr_is_tty() -> bool {
    std::io::stderr().is_terminal()
}

pub fn format_elapsed(duration: Duration) -> String {
    format!("{:.2}s", duration.as_secs_f64())
}
