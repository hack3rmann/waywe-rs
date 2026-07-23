use calloop::{
    EventSource, Interest, Mode, Poll, PostAction, Readiness, Token, TokenFactory, generic::Generic,
};
use inotify::{Inotify, WatchMask};
use std::io::ErrorKind;
use thiserror::Error;
use waywe_config::{Config, ReadConfigError};

pub struct ConfigEventSource {
    inotify: Generic<Inotify>,
    buf: Vec<u8>,
}

impl ConfigEventSource {
    pub fn new() -> Self {
        let inotify = Inotify::init().unwrap();

        for path in Config::config_paths() {
            if !path.exists() || !path.is_dir() {
                continue;
            }

            inotify
                .watches()
                .add(
                    &path,
                    WatchMask::MODIFY | WatchMask::CREATE | WatchMask::DELETE | WatchMask::MOVE,
                )
                .unwrap();
        }

        Self {
            inotify: Generic::new(inotify, Interest::READ, Mode::Level),
            buf: vec![0; 4096],
        }
    }
}

impl Default for ConfigEventSource {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum ConfigWatchError {}

impl EventSource for ConfigEventSource {
    type Event = Result<Config, ReadConfigError>;
    type Metadata = ();
    type Ret = ();
    type Error = std::io::Error;

    fn process_events<F>(
        &mut self,
        readiness: Readiness,
        token: Token,
        mut callback: F,
    ) -> Result<PostAction, Self::Error>
    where
        F: FnMut(Self::Event, &mut Self::Metadata) -> Self::Ret,
    {
        self.inotify.process_events(readiness, token, |_, inotify| {
            // Safety: inotify's Fd won't get dropped
            let inotify = unsafe { inotify.get_mut() };

            let mut events = inotify.read_events_blocking(&mut self.buf)?;

            if events.next().is_some() {
                callback(Config::read_from_config_paths(), &mut ());
            }

            // Drain remaining events
            loop {
                match inotify.read_events(&mut self.buf) {
                    Ok(_events) => continue,
                    Err(error)
                        if matches!(
                            error.kind(),
                            ErrorKind::UnexpectedEof | ErrorKind::WouldBlock
                        ) =>
                    {
                        break;
                    }
                    Err(other) => return Err(other),
                }
            }

            Ok(PostAction::Continue)
        })
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
        self.inotify.reregister(poll, token_factory)
    }

    fn unregister(&mut self, poll: &mut Poll) -> calloop::Result<()> {
        self.inotify.unregister(poll)
    }
}
