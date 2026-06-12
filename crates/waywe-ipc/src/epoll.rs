use rustix::{
    buffer::spare_capacity,
    event::epoll::{self, Event, EventData, EventFlags},
    fs::Timespec,
    io::Errno,
};
use std::{
    os::fd::{AsFd, AsRawFd, OwnedFd},
    time::Duration,
};

pub struct Epoll {
    fd: OwnedFd,
}

impl Epoll {
    pub fn new<F: AsFd + AsRawFd>(fds: impl IntoIterator<Item = F>) -> Result<Self, Errno> {
        let epoll_fd = epoll::create(epoll::CreateFlags::CLOEXEC)?;

        for fd in fds {
            epoll::add(
                &epoll_fd,
                fd.as_fd(),
                EventData::new_u64(fd.as_raw_fd() as u64),
                EventFlags::IN,
            )?;
        }

        Ok(Self { fd: epoll_fd })
    }

    pub fn wait(&self, buf: &mut PolledFds, timeout: Option<Duration>) -> Result<(), Errno> {
        let wait_time = timeout.and_then(|t| Timespec::try_from(t).ok());

        buf.clear();

        epoll::wait(
            &self.fd,
            spare_capacity(&mut buf.events),
            wait_time.as_ref(),
        )?;

        Ok(())
    }
}

pub struct PolledFds {
    events: Vec<Event>,
}

impl PolledFds {
    pub fn new() -> Self {
        Self::with_capacity(1)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
        }
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn contains(&self, fd: impl AsFd) -> bool {
        self.events
            .iter()
            .any(|event| event.data.u64() as i32 == fd.as_fd().as_raw_fd())
    }

    pub fn count_of(&self, fd: impl AsFd) -> usize {
        self.events
            .iter()
            .filter(|event| event.data.u64() as i32 == fd.as_fd().as_raw_fd())
            .count()
    }
}

impl Default for PolledFds {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<Event>> for PolledFds {
    fn from(events: Vec<Event>) -> Self {
        Self { events }
    }
}
