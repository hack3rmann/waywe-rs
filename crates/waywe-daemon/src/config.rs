use calloop::{
    EventSource, Interest, Mode, Poll, PostAction, Readiness, Token, TokenFactory,
    generic::Generic,
    timer::{TimeoutAction, Timer},
};
use inotify::{Inotify, WatchMask};
use std::{
    io::{self, ErrorKind},
    time::Duration,
};
use thiserror::Error;
use waywe_config::{Config, DhallErrorSource, ReadConfigError};

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    DoNothing,
    RegisterTimer,
    UnregisterTimer,
}

pub struct ConfigEventSource {
    debounce_duration: Duration,
    state: State,
    inotify: Generic<Inotify>,
    timer: Option<Timer>,
    buf: Vec<u8>,
}

impl ConfigEventSource {
    pub fn new(debounce_duration: Duration) -> Self {
        Self {
            debounce_duration,
            ..Self::default()
        }
    }
}

impl Default for ConfigEventSource {
    fn default() -> Self {
        let inotify = Inotify::init().unwrap();

        for path in Config::config_paths() {
            if !path.exists() || !path.is_dir() {
                continue;
            }

            inotify
                .watches()
                .add(
                    &path,
                    WatchMask::MODIFY | WatchMask::CREATE | WatchMask::MOVE,
                )
                .unwrap();
        }

        Self {
            debounce_duration: Duration::from_millis(200),
            state: State::DoNothing,
            inotify: Generic::new(inotify, Interest::READ, Mode::Level),
            timer: None,
            buf: vec![0; 4096],
        }
    }
}

fn loop_read_config(mut n_tries: usize) -> Result<Config, ReadConfigError> {
    let is_enoent = |related: &[DhallErrorSource]| -> bool {
        related.iter().all(|error| matches!(&error.error, serde_dhall::Error::Dhall(dhall::error::Error::Io(error)) if error.kind() == ErrorKind::NotFound))
    };

    loop {
        let error;

        match Config::read_from_config_paths() {
            Ok(config) => break Ok(config),
            Err(ReadConfigError::ConfigUnreachable { related }) if is_enoent(&related) => {
                error = ReadConfigError::ConfigUnreachable { related };
                n_tries -= 1;
            }
            Err(other) => break Err(other),
        }

        if n_tries <= 1 {
            break Err(error);
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigWatchError {}

impl EventSource for ConfigEventSource {
    type Event = Result<Config, ReadConfigError>;
    type Metadata = ();
    type Ret = ();
    type Error = io::Error;

    fn process_events<F>(
        &mut self,
        readiness: Readiness,
        token: Token,
        mut callback: F,
    ) -> Result<PostAction, Self::Error>
    where
        F: FnMut(Self::Event, &mut Self::Metadata) -> Self::Ret,
    {
        let generic_action = self
            .inotify
            .process_events(readiness, token, |_, inotify| {
                let mut action = PostAction::Continue;

                // Safety: inotify's Fd won't get dropped
                let inotify = unsafe { inotify.get_mut() };

                let mut events = inotify.read_events_blocking(&mut self.buf)?;

                if events.next().is_some() {
                    self.state = State::RegisterTimer;
                    action = PostAction::Reregister;
                }

                // Drain remaining events
                loop {
                    match inotify.read_events(&mut self.buf) {
                        Ok(_events) => continue,
                        Err(error)
                            if let ErrorKind::UnexpectedEof | ErrorKind::WouldBlock =
                                error.kind() =>
                        {
                            break;
                        }
                        Err(other) => return Err(other),
                    }
                }

                Ok(action)
            })?;

        if let Some(timer) = &mut self.timer {
            let timer_action = timer.process_events(readiness, token, |_instant, &mut ()| {
                callback(loop_read_config(5), &mut ());
                TimeoutAction::Drop
            })?;

            if timer_action == PostAction::Remove {
                self.state = State::UnregisterTimer;
                return Ok(PostAction::Reregister);
            }
        }

        Ok(generic_action)
    }

    fn register(
        &mut self,
        poll: &mut Poll,
        token_factory: &mut TokenFactory,
    ) -> calloop::Result<()> {
        self.inotify.register(poll, token_factory)
    }

    fn reregister(
        &mut self,
        poll: &mut Poll,
        token_factory: &mut TokenFactory,
    ) -> calloop::Result<()> {
        match self.state {
            State::RegisterTimer => {
                if let Some(mut timer) = self.timer.take() {
                    timer.unregister(poll)?;
                }

                let mut timer = Timer::from_duration(self.debounce_duration);
                timer.register(poll, token_factory)?;
                self.timer = Some(timer);

                self.state = State::DoNothing;
            }
            State::UnregisterTimer => {
                if let Some(mut timer) = self.timer.take() {
                    timer.unregister(poll)?;
                }
            }
            State::DoNothing => {}
        }

        self.inotify.reregister(poll, token_factory)
    }

    fn unregister(&mut self, poll: &mut Poll) -> calloop::Result<()> {
        if let Some(timer) = &mut self.timer {
            timer.unregister(poll)?;
        }

        self.inotify.unregister(poll)
    }
}
