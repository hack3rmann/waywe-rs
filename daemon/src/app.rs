use crate::{
    event::EventEmitter,
    event_loop::{App, Event, FrameError, FrameInfo},
    runtime::Runtime,
    wallpaper::{
        self, DynWallpaper, IntoDynWallpaper, RenderState, RequiredFeaturesExt as _,
        transition::{TransitionConfig, TransitionWallpaper},
    },
};
use glam::Vec2;
use runtime::{config::Config, profile::SetupProfile};
use smallvec::SmallVec;
use std::{iter, sync::Arc, time::Duration};
use tracing::error;

#[derive(Default)]
pub struct VideoApp {
    pub wallpapers: SmallVec<[Option<DynWallpaper>; 1]>,
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
        monitor_index: usize,
    ) {
        let wallpaper = wallpaper.into_dyn_wallpaper();

        let centre = self.config.animation.center_position.get();
        let aspect_ratio = runtime
            .wayland
            .client_state
            .aspect_ratio(monitor_index)
            .unwrap();

        let config = TransitionConfig {
            duration: Duration::from_millis(self.config.animation.duration_milliseconds),
            direction: self.config.animation.direction,
            interpolation: self.config.animation.easing,
            centre: Vec2::new(centre.x * aspect_ratio, centre.y),
        };

        self.wallpapers[monitor_index] = Some(match self.wallpapers[monitor_index].take() {
            None => wallpaper,
            Some(from) => TransitionWallpaper::new(runtime, from, wallpaper, config, monitor_index)
                .into_dyn_wallpaper(),
        });
    }

    pub fn resolve_transitions(&mut self) {
        for wallpaper in self.wallpapers.iter_mut() {
            if let Some(unresolved_wallpaper) = wallpaper.take() {
                *wallpaper = Some(TransitionWallpaper::try_resolve_any(unresolved_wallpaper));
            }
        }
    }
}

pub enum VideoAppEvent {
    WallpaperPrepared {
        wallpaper: DynWallpaper,
        monitor_index: usize,
    },
    Error(Box<dyn std::error::Error + Send + 'static>),
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
                monitor_index,
            }) => {
                runtime.control_flow.busy();
                self.set_wallpaper(runtime, wallpaper, monitor_index);
            }
            Event::ResetMonitors { count } => {
                self.wallpapers = iter::from_fn(|| Some(None)).take(count).collect();
            }
            Event::Custom(_custom) => {}
            Event::NewWallpaper { path, ty } => {
                runtime.enable(ty.required_features()).await;

                for monitor_index in 0..self.wallpapers.len() {
                    let path = path.clone();
                    let gpu = Arc::clone(&runtime.wgpu);
                    let monitor_size = runtime
                        .wayland
                        .client_state
                        .monitor_size(monitor_index)
                        .unwrap();

                    if let Err(error) = SetupProfile::new(&path, ty, monitor_size).store() {
                        error!(?error, "failed to save setup profile");
                    }

                    runtime
                        .task_pool
                        .spawn(move |mut emitter: EventEmitter<VideoAppEvent>| {
                            let event = match wallpaper::create(
                                &gpu,
                                monitor_size,
                                &path,
                                ty,
                                monitor_index,
                            ) {
                                Ok(wallpaper) => VideoAppEvent::WallpaperPrepared {
                                    wallpaper,
                                    monitor_index,
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

        for monitor_index in 0..self.wallpapers.len() {
            let Some(wallpaper) = self.wallpapers[monitor_index].as_mut() else {
                continue;
            };

            if !self.do_force_frame && wallpaper.render_state() == RenderState::Done {
                continue;
            }

            let surface_texture = runtime.wgpu.surfaces[monitor_index]
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
