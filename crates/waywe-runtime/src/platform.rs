use calloop::{EventIterator, EventSource, Poll, PostAction, Readiness, Token, TokenFactory};
use glam::UVec2;
use smallstr::SmallString;
use static_assertions::assert_obj_safe;
use std::{collections::BTreeMap, error::Error, fmt, num::NonZeroU32};
use wgpu::SurfaceTarget;

pub type MonitorMap<T> = BTreeMap<MonitorName, T>;
pub type MonitorName = SmallString<[u8; 24]>;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Scale(NonZeroU32);

impl Scale {
    pub const ONE: Self = Self(NonZeroU32::new(120).unwrap());

    pub const fn new(frac_120: u32) -> Self {
        Self(match NonZeroU32::new(frac_120) {
            Some(value) => value,
            None => NonZeroU32::new(120).unwrap(),
        })
    }

    pub const fn value(self) -> f32 {
        self.0.get() as f32 / 120.0
    }

    pub fn to_phisical(self, logical_size: UVec2) -> UVec2 {
        self.0.get() * logical_size / 120
    }

    pub fn to_logical(self, phisical_size: UVec2) -> UVec2 {
        120 * phisical_size / self.0.get()
    }
}

impl Default for Scale {
    fn default() -> Self {
        Self::ONE
    }
}

impl fmt::Debug for Scale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/120", self.0)
    }
}

pub trait WaywePlatform: Send + Sync + 'static {
    fn get_surface(&self, monitor_name: &str) -> SurfaceTarget<'static>;
    fn event_source(&self) -> Box<dyn PlatformEventSource>;
    fn drain_stored_events(&self, handle: &mut dyn FnMut(PlatformEvent));
}
assert_obj_safe!(WaywePlatform);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct SurfaceInfo {
    pub monitor_name: MonitorName,
    pub phisical_size: UVec2,
    pub scale: Scale,
}

impl SurfaceInfo {
    pub fn logical_size(&self) -> UVec2 {
        self.scale.to_logical(self.phisical_size)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlatformEvent {
    ResizeRequested {
        monitor_name: MonitorName,
        phisical_size: UVec2,
    },
    MonitorPlugged {
        info: SurfaceInfo,
    },
    MonitorUnplugged {
        monitor_name: MonitorName,
    },
    CursorMoved {
        position: UVec2,
    },
}

pub trait PlatformEventSource {
    fn process_events(
        &mut self,
        readiness: Readiness,
        token: Token,
        callback: &mut dyn FnMut(PlatformEvent),
    ) -> Result<PostAction, Box<dyn Error + Send + Sync>>;

    fn register(
        &mut self,
        poll: &mut Poll,
        token_factory: &mut TokenFactory,
    ) -> calloop::Result<()>;

    fn reregister(
        &mut self,
        poll: &mut Poll,
        token_factory: &mut TokenFactory,
    ) -> calloop::Result<()>;

    fn unregister(&mut self, poll: &mut Poll) -> calloop::Result<()>;

    fn before_sleep(&mut self) -> calloop::Result<Option<(Readiness, Token)>>;

    fn before_handle_events(&mut self, events: EventIterator<'_>);
}
assert_obj_safe!(PlatformEventSource);

impl EventSource for Box<dyn PlatformEventSource> {
    type Event = PlatformEvent;
    type Metadata = ();
    type Ret = ();
    type Error = Box<dyn Error + Send + Sync>;

    const NEEDS_EXTRA_LIFECYCLE_EVENTS: bool = true;

    fn process_events<F>(
        &mut self,
        readiness: Readiness,
        token: Token,
        mut callback: F,
    ) -> Result<PostAction, Self::Error>
    where
        F: FnMut(Self::Event, &mut Self::Metadata) -> Self::Ret,
    {
        PlatformEventSource::process_events(self.as_mut(), readiness, token, &mut move |event| {
            callback(event, &mut ())
        })
    }

    fn register(
        &mut self,
        poll: &mut Poll,
        token_factory: &mut TokenFactory,
    ) -> calloop::Result<()> {
        PlatformEventSource::register(self.as_mut(), poll, token_factory)
    }

    fn reregister(
        &mut self,
        poll: &mut Poll,
        token_factory: &mut TokenFactory,
    ) -> calloop::Result<()> {
        PlatformEventSource::reregister(self.as_mut(), poll, token_factory)
    }

    fn unregister(&mut self, poll: &mut Poll) -> calloop::Result<()> {
        PlatformEventSource::unregister(self.as_mut(), poll)
    }

    fn before_sleep(&mut self) -> calloop::Result<Option<(Readiness, Token)>> {
        PlatformEventSource::before_sleep(self.as_mut())
    }

    fn before_handle_events(&mut self, events: EventIterator<'_>) {
        PlatformEventSource::before_handle_events(self.as_mut(), events);
    }
}
