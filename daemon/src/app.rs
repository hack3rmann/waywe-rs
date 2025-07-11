use crate::{
    almost::Almost,
    event::Event,
    event_loop::{App, FrameError, FrameInfo, SetWallpaper},
    runtime::{
        Runtime,
        wayland::{MonitorId, MonitorMap, WaylandEvent},
    },
    wallpaper::{
        self, DynWallpaper, IntoDynWallpaper, RenderState, RequiredFeaturesExt as _,
        transition::{TransitionConfig, TransitionWallpaper},
    },
};
use glam::Vec2;
use runtime::{
    WallpaperType,
    config::Config,
    profile::{Monitor, SetupProfile},
};
use smallvec::{SmallVec, smallvec};
use std::{path::PathBuf, sync::Arc, time::Duration};
use tracing::{debug, error};

#[derive(Default)]
pub struct VideoApp {
    pub wallpapers: MonitorMap<DynWallpaper>,
    pub config: Config,
    pub do_force_frame: bool,
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
        runtime: &Runtime,
        wallpaper: impl IntoDynWallpaper,
        monitor_id: MonitorId,
    ) {
        let wallpaper = wallpaper.into_dyn_wallpaper();

        let centre = self.config.animation.center_position.get();
        let aspect_ratio = runtime
            .wayland
            .client_state
            .aspect_ratio(monitor_id)
            .unwrap();

        let config = TransitionConfig {
            duration: Duration::from_millis(self.config.animation.duration_milliseconds),
            direction: self.config.animation.direction,
            interpolation: self.config.animation.easing,
            centre: Vec2::new(centre.x * aspect_ratio, centre.y),
        };

        let wallpaper = match self.wallpapers.remove(&monitor_id) {
            None => wallpaper,
            Some(from) => TransitionWallpaper::new(runtime, from, wallpaper, config, monitor_id)
                .into_dyn_wallpaper(),
        };

        self.wallpapers.insert(monitor_id, wallpaper);
    }

    pub fn resolve_transitions(&mut self) {
        // FIXME(hack3rmann): really bad
        let keys = self
            .wallpapers
            .keys()
            .copied()
            .collect::<SmallVec<[_; 4]>>();

        for monitor_id in keys {
            let unresolved_wallpaper = self.wallpapers.remove(&monitor_id).unwrap();
            self.wallpapers.insert(
                monitor_id,
                TransitionWallpaper::try_resolve_any(unresolved_wallpaper),
            );
        }
    }
}

pub struct WallpaperPreparedEvent {
    pub wallpaper: DynWallpaper,
    pub monitor_id: MonitorId,
}

pub struct NewWallpaperEvent {
    pub path: PathBuf,
    pub ty: WallpaperType,
    pub set: SetWallpaper,
}

impl App for VideoApp {
    // TODO(hack3rmann): collect all handlers into a hashmap in event loop
    // HACK(hack3rmann): should write `e` (or no more than 4 characters) to avoid wierd formatting
    async fn process_event(&mut self, runtime: &mut Runtime, mut e: Event) {
        e.handle(async |event| {
            let WallpaperPreparedEvent {
                wallpaper,
                monitor_id,
            } = event;

            runtime.control_flow.busy();
            self.set_wallpaper(runtime, wallpaper, monitor_id);
        })
        .await;

        e.handle(async |NewWallpaperEvent { path, ty, set }| {
            runtime.enable(ty.required_features()).await;

            let monitor_ids: SmallVec<[MonitorId; 4]> = match set {
                SetWallpaper::ForAll => runtime
                    .wayland
                    .client_state
                    .monitors
                    .read()
                    .unwrap()
                    .keys()
                    .copied()
                    .collect(),
                SetWallpaper::ForMonitor(wl_object_id) => smallvec![wl_object_id],
            };

            for monitor_id in monitor_ids {
                let path = path.clone();
                let gpu = Arc::clone(&runtime.wgpu);
                let monitor_size = runtime
                    .wayland
                    .client_state
                    .monitor_size(monitor_id)
                    .unwrap();
                let monitor_name = runtime
                    .wayland
                    .client_state
                    .monitor_name(monitor_id)
                    .unwrap();

                if let Err(error) = SetupProfile::default()
                    .with(
                        monitor_name,
                        Monitor {
                            wallpaper_type: ty,
                            path: path.clone(),
                        },
                    )
                    .store()
                {
                    error!(?error, "failed to save setup profile");
                }

                runtime.task_pool.spawn(move |mut emitter| {
                    let event = match wallpaper::create(&gpu, monitor_size, &path, ty, monitor_id) {
                        Ok(wallpaper) => WallpaperPreparedEvent {
                            wallpaper,
                            monitor_id,
                        },
                        Err(error) => {
                            error!(?error, "failed to create wallpaper");
                            return;
                        }
                    };

                    emitter.emit(event).unwrap();
                });
            }
        })
        .await;

        e.handle(async |e: WaylandEvent| match e {
            WaylandEvent::ResizeRequested { monitor_id, size } => {
                if Almost::is_init(&runtime.wgpu) {
                    runtime.wgpu.resize_surface(monitor_id, size);
                }

                self.do_force_frame = true;
            }
            WaylandEvent::MonitorPlugged { id: monitor_id } => {
                if Almost::is_init(&runtime.wgpu) {
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
                        set: SetWallpaper::ForMonitor(monitor_id),
                    };

                    runtime.task_pool.emitter.emit(event).unwrap();
                }

                runtime.control_flow.busy();
            }
            WaylandEvent::MonitorUnplugged { id: monitor_id } => {
                debug!(?monitor_id, "unplugged a monitor");

                _ = self.wallpapers.remove(&monitor_id);
                runtime.wgpu.unregister_surface(monitor_id);
            }
        })
        .await;
    }

    async fn frame(&mut self, runtime: &mut Runtime) -> Result<FrameInfo, FrameError> {
        self.resolve_transitions();

        if Almost::is_uninit(&runtime.wgpu) {
            return Err(FrameError::NoWorkToDo);
        }

        // FIXME(hack3rmann): multiple monitors
        let mut result = Err(FrameError::NoWorkToDo);
        let surfaces = runtime.wgpu.surfaces.read().unwrap();

        for (&monitor_id, wallpaper) in self.wallpapers.iter_mut() {
            if !self.do_force_frame && wallpaper.render_state() == RenderState::Done {
                continue;
            }

            let surface_texture = surfaces[&monitor_id].get_current_texture().unwrap();
            let surface_view = surface_texture.texture.create_view(&Default::default());

            let mut encoder = runtime
                .wgpu
                .device
                .create_command_encoder(&Default::default());

            result = wallpaper.frame(runtime, &mut encoder, &surface_view);

            _ = runtime.wgpu.queue.submit([encoder.finish()]);

            surface_texture.present();
        }

        if let Err(FrameError::NoWorkToDo) = &result {
            runtime.control_flow.idle();
        }

        self.do_force_frame = false;

        result
    }
}
