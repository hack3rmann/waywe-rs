use crate::{
    event_loop::{App, Event, FrameError, FrameInfo},
    runtime::{
        Runtime,
        wayland::{MonitorId, MonitorMap},
    },
    wallpaper::{
        self, DynWallpaper, IntoDynWallpaper, RenderState, RequiredFeaturesExt as _,
        transition::{TransitionConfig, TransitionWallpaper},
    },
};
use glam::Vec2;
use runtime::{config::Config, profile::SetupProfile};
use smallvec::SmallVec;
use std::{error::Error, sync::Arc, time::Duration};
use tracing::error;

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
        runtime: &Runtime<VideoAppEvent>,
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

pub enum VideoAppEvent {
    WallpaperPrepared {
        wallpaper: DynWallpaper,
        monitor_id: MonitorId,
    },
    Error(Box<dyn Error + Send + 'static>),
}

impl App for VideoApp {
    type CustomEvent = VideoAppEvent;

    async fn process_event(
        &mut self,
        runtime: &mut Runtime<Self::CustomEvent>,
        event: Event<Self::CustomEvent>,
    ) {
        match event {
            Event::Custom(VideoAppEvent::WallpaperPrepared {
                wallpaper,
                monitor_id,
            }) => {
                dbg!("WallpaperPrepared", monitor_id);
                runtime.control_flow.busy();
                self.set_wallpaper(runtime, wallpaper, monitor_id);
            }
            Event::Custom(_custom) => {}
            Event::NewWallpaper { path, ty } => {
                runtime.enable(ty.required_features()).await;

                for &monitor_id in runtime.wayland.client_state.monitors.read().unwrap().keys() {
                    let path = path.clone();
                    let gpu = Arc::clone(&runtime.wgpu);
                    let monitor_size = runtime
                        .wayland
                        .client_state
                        .monitor_size(monitor_id)
                        .unwrap();

                    if let Err(error) = SetupProfile::new(&path, ty, monitor_size).store() {
                        error!(?error, "failed to save setup profile");
                    }

                    runtime.task_pool.spawn(move |mut emitter| {
                        let event =
                            match wallpaper::create(&gpu, monitor_size, &path, ty, monitor_id) {
                                Ok(wallpaper) => VideoAppEvent::WallpaperPrepared {
                                    wallpaper,
                                    monitor_id,
                                },
                                Err(error) => VideoAppEvent::Error(Box::new(error)),
                            };

                        emitter.emit(event).unwrap();
                    });
                }
            }
            Event::ResizeRequested { size } => {
                runtime.wgpu.resize_surface(size);
                self.do_force_frame = true;
            }
        }
    }

    async fn frame(
        &mut self,
        runtime: &mut Runtime<Self::CustomEvent>,
    ) -> Result<FrameInfo, FrameError> {
        self.resolve_transitions();

        // FIXME(hack3rmann): multiple monitors
        let mut result = Err(FrameError::NoWorkToDo);

        for (&monitor_id, wallpaper) in self.wallpapers.iter_mut() {
            if !self.do_force_frame && wallpaper.render_state() == RenderState::Done {
                continue;
            }

            let surface_texture = runtime.wgpu.surfaces[&monitor_id]
                .get_current_texture()
                .unwrap();
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
