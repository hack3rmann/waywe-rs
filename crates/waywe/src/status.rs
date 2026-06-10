use anstream::AutoStream;
use anstyle::{AnsiColor, Effects, Style};
use std::{fmt::Display, io::Write, time::Duration};

const HEADER: Style = AnsiColor::BrightGreen.on_default().effects(Effects::BOLD);

pub fn status(label: &str, message: impl Display) {
    let mut stderr = AutoStream::auto(std::io::stderr());
    let _ = write!(stderr, "{HEADER}{label:>12}{HEADER:#} {message}\n");
}

pub fn format_elapsed(duration: Duration) -> String {
    format!("{:.2}s", duration.as_secs_f64())
}
