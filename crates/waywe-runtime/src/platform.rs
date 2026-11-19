use calloop::{EventIterator, EventSource, Poll, PostAction, Readiness, Token, TokenFactory};
use glam::UVec2;
use smallstr::SmallString;
use static_assertions::assert_obj_safe;
use std::{collections::BTreeMap, error::Error, fmt, num::NonZeroU32};

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

/// Result of a call to [`Surface::get_current_texture`].
///
/// See variant documentation for how to handle each case.
pub enum CurrentSurfaceTexture {
    /// Successfully acquired a surface texture with no issues.
    Success(Box<dyn SurfaceTexture>),
    /// Successfully acquired a surface texture, but texture no longer matches the properties of the underlying surface.
    /// It's highly recommended to call [`Surface::configure`] again for optimal performance.
    Suboptimal(Box<dyn SurfaceTexture>),
    /// A timeout was encountered while trying to acquire the next frame.
    ///
    /// Applications should skip the current frame and try again later.
    Timeout,
    /// The window is occluded (e.g. minimized or behind another window).
    ///
    /// Applications should skip the current frame and try again once the window
    /// is no longer occluded.
    Occluded,
    /// The underlying surface has changed, and therefore the surface configuration is outdated.
    ///
    /// Call [`Surface::configure()`] and try again.
    Outdated,
    /// The surface has been lost and needs to be recreated.
    ///
    /// If the device as a whole is lost (see [`set_device_lost_callback()`][crate::Device::set_device_lost_callback]), then
    /// you need to recreate the device and all resources.
    /// Otherwise, call [`Instance::create_surface()`] to recreate the surface,
    /// then [`Surface::configure()`], and try again.
    Lost,
    /// A validation error inside [`Surface::get_current_texture()`] was raised
    /// and caught by an [error scope](crate::Device::push_error_scope) or
    /// [`on_uncaptured_error()`][crate::Device::on_uncaptured_error].
    ///
    /// Applications should attend to the validation error and try again.
    Validation,
}

pub trait SurfaceTexture {
    fn texture(&self) -> &wgpu::Texture;
    fn present(self: Box<Self>, queue: &wgpu::Queue);
}
assert_obj_safe!(SurfaceTexture);

pub trait WayweSurface: Send + Sync {
    fn get_current_texture(&self) -> CurrentSurfaceTexture;
    fn configure(&self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration);
    fn get_format(&self, adapter: &wgpu::Adapter) -> wgpu::TextureFormat;
    fn get_config(
        &self,
        adapter: &wgpu::Adapter,
        physical_size: UVec2,
    ) -> wgpu::SurfaceConfiguration;
}
assert_obj_safe!(WayweSurface);

pub trait WaywePlatform: Send + Sync + 'static {
    fn create_surface(
        &self,
        instance: &wgpu::Instance,
        monitor_name: &str,
    ) -> Box<dyn WayweSurface>;
    fn event_source(&self) -> Box<dyn PlatformEventSource>;
    fn drain_stored_events(&self, handle: &mut dyn FnMut(PlatformEvent));
}
assert_obj_safe!(WaywePlatform);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct SurfaceInfo {
    pub monitor_name: MonitorName,
    pub physical_size: UVec2,
    pub scale: Scale,
}

impl SurfaceInfo {
    pub fn logical_size(&self) -> UVec2 {
        self.scale.to_logical(self.physical_size)
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
