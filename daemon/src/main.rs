pub mod almost;
pub mod event_loop;
pub mod runtime;
pub mod video_pipeline;

use almost::Almost;
use event_loop::{App, Event, EventLoop, FrameError, FrameInfo};
use runtime::Runtime;
use std::time::Duration;
use tracing::error;
use video::{
    BackendError, Codec, CodecContext, FormatContext, Frame, FrameDuration, MediaType, Packet,
    RatioI32, VideoPixelFormat,
};
use video_pipeline::VideoPipeline;

struct Video {
    pub pipeline: VideoPipeline,
    pub format_context: FormatContext,
    pub time_base: RatioI32,
    pub best_stream_index: usize,
    pub codec_context: CodecContext,
    pub frame_time_fallback: Duration,
}

struct VideoApp {
    pub do_loop_video: bool,
    pub video: Almost<Video>,
    pub packet: Option<Packet>,
    pub frame: Frame,
}

impl Default for VideoApp {
    fn default() -> Self {
        Self {
            do_loop_video: true,
            video: Almost::uninit(),
            packet: None,
            frame: Frame::new(),
        }
    }
}

impl App for VideoApp {
    async fn process_event(&mut self, runtime: &mut Runtime, event: Event) {
        match event {
            Event::UpdateWallpaper { path } => {
                runtime.init_wgpu().await;
                runtime.init_video();

                let format_context = match FormatContext::from_input(&path) {
                    Ok(context) => context,
                    Err(error) => {
                        error!(?error, ?path, "failed to open file");
                        return;
                    }
                };

                let best_stream = match format_context.find_best_stream(MediaType::Video) {
                    Ok(stream) => stream,
                    Err(error) => {
                        error!(?error, ?path, "failed to find video stream");
                        return;
                    }
                };

                let time_base = best_stream.time_base();
                let best_stream_index = best_stream.index();
                let codec_parameters = best_stream.codec_parameters();
                let frame_rate = codec_parameters.frame_rate().unwrap();
                let video_size = codec_parameters.video_size().unwrap();

                if !matches!(
                    codec_parameters.format(),
                    Some(video::AudioVideoFormat::Video(VideoPixelFormat::Yuv420p))
                ) {
                    error!(format = ?codec_parameters.format(), "invalid_video_format");
                    return;
                }

                let mut codec_context = match CodecContext::from_parameters(codec_parameters) {
                    Ok(context) => context,
                    Err(error) => {
                        error!(?error, "failed to construct codec context");
                        return;
                    }
                };

                let Some(decoder) = Codec::find_for_id(codec_context.codec_id()) else {
                    error!("failed to find decoder");
                    return;
                };

                if let Err(error) = codec_context.open(decoder) {
                    error!(?error, "failed to open codec context");
                }

                const FRAME_DURATION_60_FPS: Duration =
                    RatioI32::new(1, 60).unwrap().to_duration_seconds();

                let frame_time_fallback = match frame_rate.inv() {
                    Some(duration) => duration.to_duration_seconds(),
                    None => FRAME_DURATION_60_FPS,
                };

                Almost::init(
                    &mut self.video,
                    Video {
                        pipeline: VideoPipeline::new(
                            &runtime.wgpu.device,
                            runtime.wgpu.surface_format,
                            video_size,
                            runtime.wayland.client_state.monitor_size(),
                        ),
                        format_context,
                        time_base,
                        best_stream_index,
                        codec_context,
                        frame_time_fallback,
                    },
                );
            }
        }
    }

    async fn frame(&mut self, runtime: &mut Runtime) -> Result<FrameInfo, FrameError> {
        if Almost::is_uninit(&self.video) {
            return Err(FrameError::Skip);
        }

        loop {
            if self.packet.is_none() {
                let packet = loop {
                    let packet = match self.video.format_context.read_packet() {
                        Ok(packet) => packet,
                        Err(BackendError::EOF) => {
                            if !self.do_loop_video {
                                return Err(FrameError::StopRequested);
                            }

                            let best_index = self.video.best_stream_index;

                            if let Err(error) = self.video.format_context.repeat_stream(best_index)
                            {
                                error!(?error, "failed to reapead video stream");
                                return Err(FrameError::Skip);
                            }

                            continue;
                        }
                        Err(error) => {
                            error!(?error, "failed to read next video packet");
                            return Err(FrameError::Skip);
                        }
                    };

                    if packet.stream_index() == self.video.best_stream_index {
                        break packet;
                    }
                };

                if let Err(error) = self.video.codec_context.send_packet(&packet) {
                    error!(?error, "failed to send packet to the decoder");
                    return Err(FrameError::Skip);
                }

                _ = self.packet.insert(packet);
            }

            match self.video.codec_context.receive_frame(&mut self.frame) {
                Ok(()) => break,
                Err(..) => {
                    self.packet = None;
                    continue;
                }
            }
        }

        let target_frame_time = self
            .frame
            .duration_in(self.video.time_base)
            .map(FrameDuration::to_duration)
            .unwrap_or(self.video.frame_time_fallback);

        let data_planes = unsafe { [self.frame.data(0), self.frame.data(1), self.frame.data(2)] };

        self.video
            .pipeline
            .write_video_frame(&runtime.wgpu.queue, data_planes);
        runtime.wgpu.queue.submit([]);

        let surface_texture = runtime.wgpu.surface.get_current_texture().unwrap();
        let surface_view = surface_texture.texture.create_view(&Default::default());

        let mut encoder = runtime
            .wgpu
            .device
            .create_command_encoder(&Default::default());

        self.video.pipeline.render(&mut encoder, &surface_view);

        _ = runtime.wgpu.queue.submit([encoder.finish()]);

        surface_texture.present();

        Ok(FrameInfo { target_frame_time })
    }
}

fn main() {
    EventLoop::<VideoApp>::default().run();
}
