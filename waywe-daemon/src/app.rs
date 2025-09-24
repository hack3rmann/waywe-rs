use crate::{
    event::{EventHandler, Handle},
    event_loop::{App, FrameError, FrameInfo, WallpaperTarget},
    runtime::{
        Runtime, RuntimeFeatures,
        wayland::{MonitorId, MonitorMap, WaylandEvent},
    },
    wallpaper::{
        self,
        scene::{cursor::CursorMoved, wallpaper::PreparedWallpaper},
        transition::{AnimationState, WallpaperTransitionPipeline, WallpaperTransitionState},
    },
};
use for_sure::prelude::*;
use glam::Vec2;
use runtime::{
    WallpaperType,
    config::Config,
    profile::{Monitor, SetupProfile},
};
use smallvec::{SmallVec, smallvec};
use std::{collections::btree_map::Entry, path::PathBuf, sync::Arc};
use tracing::{debug, error};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WallpaperState {
    #[default]
    Running,
    Paused,
}

impl WallpaperState {
    pub const fn inverted(self) -> Self {
        match self {
            Self::Running => Self::Paused,
            Self::Paused => Self::Running,
        }
    }

    pub const fn is_paused(self) -> bool {
        matches!(self, Self::Paused)
    }

    pub const fn is_running(self) -> bool {
        matches!(self, Self::Running)
    }
}

#[derive(Default)]
pub struct VideoApp {
    pub wallpapers: MonitorMap<Vec<PreparedWallpaper>>,
    pub wallpaper_states: MonitorMap<WallpaperState>,
    pub config: Config,
    pub do_force_frame: bool,
    pub transition_states: MonitorMap<WallpaperTransitionState>,
    pub transition_pipelines: MonitorMap<WallpaperTransitionPipeline>,
}

impl VideoApp {
    pub fn from_config(config: Config) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    pub fn set_wallpaper(
        &mut self,
        _: &Runtime,
        wallpaper: PreparedWallpaper,
        monitor_id: MonitorId,
    ) {
        match self.wallpapers.entry(monitor_id) {
            Entry::Vacant(entry) => _ = entry.insert(vec![wallpaper]),
            Entry::Occupied(mut occupied_entry) => occupied_entry.get_mut().push(wallpaper),
        }

        self.wallpaper_states
            .insert(monitor_id, WallpaperState::Running);
    }
}

pub struct WallpaperPreparedEvent {
    pub wallpaper: PreparedWallpaper,
    pub monitor_id: MonitorId,
}

pub struct NewWallpaperEvent {
    pub path: PathBuf,
    pub ty: WallpaperType,
    pub target: WallpaperTarget,
}

pub struct WallpaperPauseEvent {
    pub target: WallpaperTarget,
}

impl App for VideoApp {
    fn populate_handler(&mut self, handler: &mut EventHandler<Self>)
    where
        Self: Sized,
    {
        handler
            .add_event::<WaylandEvent>()
            .add_event::<NewWallpaperEvent>()
            .add_event::<WallpaperPreparedEvent>()
            .add_event::<WallpaperPauseEvent>();
    }

    async fn frame(&mut self, runtime: &mut Runtime) -> Result<FrameInfo, FrameError> {
        if Almost::is_nil(&runtime.wgpu) {
            return Err(FrameError::NoWorkToDo);
        }

        // FIXME(hack3rmann): multiple monitors
        let mut result = Err(FrameError::NoWorkToDo);

        for (&monitor_id, wallpapers) in self.wallpapers.iter_mut() {
            if let Some(&state) = self.wallpaper_states.get(&monitor_id)
                && state.is_paused()
            {
                continue;
            }

            // FIXME(hack3rmann): move to initialization
            let transition_pipeline = self
                .transition_pipelines
                .entry(monitor_id)
                .or_insert_with(|| WallpaperTransitionPipeline::new(&runtime.wgpu, monitor_id));
            let state = self.transition_states.entry(monitor_id).or_insert_with(|| {
                WallpaperTransitionState::new(&runtime.wgpu, transition_pipeline)
            });

            let surfaces = runtime.wgpu.surfaces.read().unwrap();
            let texture = surfaces[&monitor_id].surface.get_current_texture().unwrap();
            let surface_view = texture.texture.create_view(&Default::default());
            let mut encoder = runtime
                .wgpu
                .device
                .create_command_encoder(&Default::default());

            match wallpapers.as_mut_slice() {
                [wallpaper] => {
                    result = wallpaper.frame(surface_view, &mut encoder);
                }
                [from, to] => {
                    _ = from.frame(state.from.clone(), &mut encoder);
                    result = to.frame(state.to.clone(), &mut encoder);

                    let animation_state = AnimationState {
                        centre: Vec2::ZERO,
                        radius: 0.5,
                        direction: 1.0,
                    };

                    transition_pipeline.render(
                        state,
                        &surface_view,
                        &mut encoder,
                        &animation_state,
                    );
                }
                _ => todo!(),
            }

            runtime.wgpu.queue.submit([encoder.finish()]);
            texture.present();
        }

        if let Err(FrameError::NoWorkToDo) = &result {
            runtime.control_flow.idle();
        }

        self.do_force_frame = false;

        result
    }
}

impl Handle<WallpaperPauseEvent> for VideoApp {
    async fn handle(&mut self, runtime: &mut Runtime, event: WallpaperPauseEvent) {
        let WallpaperPauseEvent { target } = event;

        let monitor_ids: SmallVec<[MonitorId; 4]> = match target {
            WallpaperTarget::ForAll => {
                let monitors = runtime.wayland.client_state.monitors.read().unwrap();
                monitors.keys().copied().collect()
            }
            WallpaperTarget::ForMonitor(id) => smallvec![id],
        };

        for monitor_id in monitor_ids {
            let Some(state) = self.wallpaper_states.get_mut(&monitor_id) else {
                continue;
            };

            *state = state.inverted();
        }
    }
}

impl Handle<WallpaperPreparedEvent> for VideoApp {
    async fn handle(&mut self, runtime: &mut Runtime, event: WallpaperPreparedEvent) {
        let WallpaperPreparedEvent {
            wallpaper,
            monitor_id,
        } = event;

        runtime.control_flow.busy();
        self.set_wallpaper(runtime, wallpaper, monitor_id);
    }
}

impl Handle<WaylandEvent> for VideoApp {
    async fn handle(&mut self, runtime: &mut Runtime, event: WaylandEvent) {
        match event {
            WaylandEvent::ResizeRequested { monitor_id, size } => {
                if Almost::is_value(&runtime.wgpu) {
                    runtime.wgpu.resize_surface(monitor_id, size);
                }

                self.do_force_frame = true;
            }
            WaylandEvent::MonitorPlugged { id: monitor_id } => {
                if Almost::is_value(&runtime.wgpu) {
                    runtime.wgpu.register_surface(&runtime.wayland, monitor_id);
                }

                let monitors = runtime.wayland.client_state.monitors.read().unwrap();
                let monitor = &monitors[&monitor_id];
                let monitor_name = Arc::clone(monitor.name.as_ref().unwrap());

                debug!(?monitor_id, ?monitor_name, "new monitor detected");

                if let Ok(mut profile) = SetupProfile::read()
                    && let Some(info) = profile.monitors.remove(&monitor_name)
                {
                    let event = NewWallpaperEvent {
                        path: info.path,
                        ty: info.wallpaper_type,
                        target: WallpaperTarget::ForMonitor(monitor_id),
                    };

                    runtime.task_pool.emitter.emit(event).unwrap();
                }

                runtime.control_flow.busy();
            }
            WaylandEvent::MonitorUnplugged { id: monitor_id } => {
                debug!(?monitor_id, "unplugged a monitor");

                _ = self.wallpapers.remove(&monitor_id);
                _ = self.wallpaper_states.remove(&monitor_id);

                runtime.wgpu.unregister_surface(monitor_id);
            }
            WaylandEvent::CursorMoved { position } => {
                for wallpaper in self.wallpapers.values_mut() {
                    // FIXME:
                    wallpaper[0]
                        .wallpaper
                        .main
                        .world
                        .trigger(CursorMoved { position });
                }
            }
        }
    }
}

impl Handle<NewWallpaperEvent> for VideoApp {
    async fn handle(&mut self, runtime: &mut Runtime, event: NewWallpaperEvent) {
        let NewWallpaperEvent { path, ty, target } = event;

        // FIXME(hack3rmann): remove runtime features
        runtime.enable(RuntimeFeatures::GPU).await;

        let monitor_ids: SmallVec<[MonitorId; 4]> = match target {
            WallpaperTarget::ForAll => {
                let monitors = runtime.wayland.client_state.monitors.read().unwrap();
                monitors.keys().copied().collect()
            }
            WallpaperTarget::ForMonitor(id) => smallvec![id],
        };

        for monitor_id in monitor_ids {
            let path = path.clone();
            let gpu = Arc::clone(&runtime.wgpu);
            let wayland = Arc::clone(&runtime.wayland);

            let monitors = runtime.wayland.client_state.monitors.read().unwrap();
            let monitor = &monitors[&monitor_id];
            let monitor_name = Arc::clone(monitor.name.as_ref().unwrap());
            let monitor_profile = Monitor {
                wallpaper_type: ty,
                path: path.clone(),
            };

            if let Err(error) = SetupProfile::default()
                .with(monitor_name, monitor_profile)
                .store()
            {
                error!(?error, "failed to save setup profile");
            }

            runtime.task_pool.spawn(move |mut emitter| {
                let event = WallpaperPreparedEvent {
                    wallpaper: wallpaper::create(gpu, wayland, &path, ty, monitor_id),
                    monitor_id,
                };

                emitter.emit(event).unwrap();
            });
        }
    }
}
