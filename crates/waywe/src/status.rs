use anstream::AutoStream;
use anstyle::{AnsiColor, Effects, Style};
use std::{
    fmt::Display,
    io::{self, IsTerminal, Write},
};

const HEADER_STYLE: Style = AnsiColor::BrightGreen.on_default().effects(Effects::BOLD);
const TRANSIENT_STYLE: Style = AnsiColor::BrightCyan.on_default().effects(Effects::BOLD);

mod needs_clear {
    use anstream::AutoStream;
    use std::{
        io::{self, Write},
        sync::atomic::{AtomicBool, Ordering::*},
    };

    static NEEDS_CLEAR: AtomicBool = AtomicBool::new(false);

    pub fn set(needs_clear: bool) {
        NEEDS_CLEAR.store(needs_clear, Relaxed);
    }

    pub fn value() -> bool {
        NEEDS_CLEAR.load(Relaxed)
    }

    pub fn erase_line_if_needed(stderr: &mut AutoStream<io::Stderr>) {
        _ = NEEDS_CLEAR.fetch_update(SeqCst, SeqCst, |needs_clear| {
            if needs_clear {
                _ = stderr.write_all(b"\x1B[K");
            }
            Some(false)
        });
    }
}

pub fn display(label: &str, message: &impl Display) {
    let mut stderr = AutoStream::auto(io::stderr());
    needs_clear::erase_line_if_needed(&mut stderr);
    let _ = writeln!(
        stderr,
        "{HEADER_STYLE}{label:>12}{HEADER_STYLE:#} {message}"
    );
}

pub fn transient_status(label: &str) {
    let mut stderr = AutoStream::auto(io::stderr());
    needs_clear::erase_line_if_needed(&mut stderr);
    let _ = write!(stderr, "{TRANSIENT_STYLE}{label:>12}{TRANSIENT_STYLE:#} ");
}

pub fn write_progress_line(line: &impl Display) {
    let mut stderr = AutoStream::auto(io::stderr());
    let _ = write!(stderr, "{line}\r");
    needs_clear::set(true);
}

pub fn clear_progress_line() {
    if !needs_clear::value() {
        let mut stderr = AutoStream::auto(io::stderr());
        let _ = stderr.write_all(b"\x1B[K");
        needs_clear::set(false);
    }
}

pub fn stderr_is_tty() -> bool {
    io::stderr().is_terminal()
}
