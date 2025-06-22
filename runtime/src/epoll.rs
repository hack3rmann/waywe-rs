use rustix::{
    event::epoll::{self, EventData, EventFlags, EventVec},
    io::Errno,
};
use std::{
    os::fd::{AsFd, AsRawFd, OwnedFd},
    time::Duration,
};

pub struct Epoll {
    fd: OwnedFd,
    timeout: Option<Duration>,
}

impl Epoll {
    pub fn new<F: AsFd + AsRawFd>(
        fds: impl IntoIterator<Item = F>,
        timeout: Option<Duration>,
    ) -> Result<Self, Errno> {
        let epoll_fd = epoll::create(epoll::CreateFlags::CLOEXEC)?;

        for fd in fds {
            epoll::add(
                &epoll_fd,
                fd.as_fd(),
                EventData::new_u64(fd.as_raw_fd() as u64),
                EventFlags::IN,
            )?;
        }

        Ok(Self {
            fd: epoll_fd,
            timeout,
        })
    }

    pub fn wait(&self) -> Result<PolledFds, Errno> {
        let wait_time = self
            .timeout
            .and_then(|d| i32::try_from(d.as_millis()).ok())
            .unwrap_or(-1);

        let mut events = EventVec::with_capacity(1);

        epoll::wait(&self.fd, &mut events, wait_time)?;

        Ok(PolledFds::new(events))
    }
}

pub struct PolledFds {
    events: EventVec,
}

impl PolledFds {
    pub fn new(events: EventVec) -> Self {
        Self { events }
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
