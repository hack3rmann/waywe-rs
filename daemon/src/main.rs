extern crate ffmpeg_next as ffmpeg;

use bytemuck::{Pod, Zeroable};
use glam::{UVec2, Vec2};
use raw_window_handle::{HasDisplayHandle as _, RawWindowHandle, WaylandWindowHandle};
use std::{
    error::Error,
    ffi::CStr,
    mem,
    pin::pin,
    thread,
    time::{Duration, Instant},
};
use video::{
    BackendError, Codec, CodecContext, FormatContext, Frame, MediaType, Packet, RatioI32, Scaler,
    ScalerFlags, ScalerFormat, VideoPixelFormat,
};
use wayland_client::{
    WlSmallVecMessageBuffer,
    interface::{
        WlCompositorCreateSurfaceRequest, WlSurfaceCommitRequest, WlSurfaceSetBufferScaleRequest,
        ZwlrLayerShellGetLayerSurfaceRequest, ZwlrLayerShellLayer,
        ZwlrLayerSurfaceAckConfigureRequest, ZwlrLayerSurfaceAnchor,
        ZwlrLayerSurfaceConfigureEvent, ZwlrLayerSurfaceKeyboardInteractivity,
        ZwlrLayerSurfaceSetAnchorRequest, ZwlrLayerSurfaceSetExclusiveZoneRequest,
        ZwlrLayerSurfaceSetKeyboardInteractivityRequest, ZwlrLayerSurfaceSetMarginRequest,
    },
    object::{HasObjectType, WlObjectType},
    sys::{
        display::WlDisplay,
        object::{FromProxy, WlObject, WlObjectHandle, dispatch::Dispatch},
        object_storage::WlObjectStorage,
        proxy::WlProxy,
        wire::{WlMessage, WlStackMessageBuffer},
    },
};
use wgpu::{ShaderStages, util::DeviceExt as _};

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
struct ClientState;

struct Compositor;

impl HasObjectType for Compositor {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Compositor;
}

impl Dispatch for Compositor {
    type State = ClientState;
    const ALLOW_EMPTY_DISPATCH: bool = true;
}

impl FromProxy for Compositor {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

struct LayerShell;

impl HasObjectType for LayerShell {
    const OBJECT_TYPE: WlObjectType = WlObjectType::LayerShell;
}

impl Dispatch for LayerShell {
    type State = ClientState;
    const ALLOW_EMPTY_DISPATCH: bool = true;
}

impl FromProxy for LayerShell {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

struct Surface;

impl HasObjectType for Surface {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Surface;
}

impl Dispatch for Surface {
    type State = ClientState;
    const ALLOW_EMPTY_DISPATCH: bool = true;
}

impl FromProxy for Surface {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

struct LayerSurface {
    handle: WlObjectHandle<Self>,
}

impl HasObjectType for LayerSurface {
    const OBJECT_TYPE: WlObjectType = WlObjectType::LayerSurface;
}

impl Dispatch for LayerSurface {
    type State = ClientState;

    fn dispatch(
        &mut self,
        _state: &Self::State,
        storage: &mut WlObjectStorage<'_, Self::State>,
        message: WlMessage<'_>,
    ) {
        let Some(ZwlrLayerSurfaceConfigureEvent { serial, .. }) = message.as_event() else {
            return;
        };

        let mut buf = WlSmallVecMessageBuffer::<3>::new();

        self.handle.request(
            &mut buf,
            storage,
            ZwlrLayerSurfaceAckConfigureRequest { serial },
        );
    }
}

impl FromProxy for LayerSurface {
    fn from_proxy(proxy: &WlProxy) -> Self {
        Self {
            handle: WlObjectHandle::new(proxy.id()),
        }
    }
}

pub const WLR_NAMESPACE: &CStr = c"waywe-runtime";

trait SurfaceExtension {
    fn raw_window_handle(&self) -> RawWindowHandle;
}

impl SurfaceExtension for WlObject<Surface> {
    fn raw_window_handle(&self) -> RawWindowHandle {
        RawWindowHandle::Wayland(WaylandWindowHandle::new(self.proxy().as_raw()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    ffmpeg::init().unwrap();

    const SCREEN_WIDTH: u32 = 2520;
    const SCREEN_HEIGHT: u32 = 1680;

    let mut client_state = pin!(ClientState);

    let display = WlDisplay::connect(client_state.as_mut()).unwrap();

    let mut buf = WlStackMessageBuffer::new();
    let mut queue = pin!(display.take_main_queue().unwrap());

    let registry = display.create_registry(&mut buf, queue.as_mut().storage_mut());

    display.roundtrip(queue.as_mut(), client_state.as_ref());

    let compositor = registry
        .bind::<Compositor>(&mut buf, queue.as_mut().storage_mut())
        .unwrap();

    let layer_shell = registry
        .bind::<LayerShell>(&mut buf, queue.as_mut().storage_mut())
        .unwrap();

    let surface: WlObjectHandle<Surface> = compositor.create_object(
        &mut buf,
        queue.as_mut().storage_mut(),
        WlCompositorCreateSurfaceRequest,
    );

    let layer_surface: WlObjectHandle<LayerSurface> = layer_shell.create_object(
        &mut buf,
        queue.as_mut().storage_mut(),
        ZwlrLayerShellGetLayerSurfaceRequest {
            surface: surface.id(),
            output: None,
            layer: ZwlrLayerShellLayer::Background,
            namespace: WLR_NAMESPACE,
        },
    );

    layer_surface.request(
        &mut buf,
        &queue.as_ref().storage(),
        ZwlrLayerSurfaceSetAnchorRequest {
            anchor: ZwlrLayerSurfaceAnchor::all(),
        },
    );

    layer_surface.request(
        &mut buf,
        &queue.as_ref().storage(),
        ZwlrLayerSurfaceSetExclusiveZoneRequest { zone: -1 },
    );

    layer_surface.request(
        &mut buf,
        &queue.as_ref().storage(),
        ZwlrLayerSurfaceSetMarginRequest {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        },
    );

    layer_surface.request(
        &mut buf,
        &queue.as_ref().storage(),
        ZwlrLayerSurfaceSetKeyboardInteractivityRequest {
            keyboard_interactivity: ZwlrLayerSurfaceKeyboardInteractivity::None,
        },
    );

    surface.request(
        &mut buf,
        &queue.as_ref().storage(),
        WlSurfaceSetBufferScaleRequest { scale: 1 },
    );

    surface.request(&mut buf, &queue.as_ref().storage(), WlSurfaceCommitRequest);

    display.roundtrip(queue.as_mut(), client_state.as_ref());

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        flags: if cfg!(debug_assertions) {
            wgpu::InstanceFlags::DEBUG | wgpu::InstanceFlags::VALIDATION
        } else {
            wgpu::InstanceFlags::empty()
        },
        ..Default::default()
    });

    let raw_display_handle = display.display_handle().unwrap().as_raw();
    let raw_window_handle = queue.as_ref().storage().object(surface).raw_window_handle();

    let wgpu_surface = unsafe {
        instance
            .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle,
                raw_window_handle,
            })
            .unwrap()
    };

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            force_fallback_adapter: false,
            compatible_surface: Some(&wgpu_surface),
        })
        .await
        .expect("failed to request adapter");

    let (device, gpu_queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::PUSH_CONSTANTS,
                label: None,
                required_limits: adapter.limits(),
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        )
        .await
        .expect("failed to request device");

    // TODO(hack3rmann): figure out the size of the current monitor
    wgpu_surface.configure(
        &device,
        &wgpu_surface
            .get_default_config(&adapter, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap(),
    );

    // TODO(hack3rmann): use the correct surface format
    let surface_format = wgpu_surface.get_capabilities(&adapter).formats[0];

    let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Glsl {
            shader: include_str!("shaders/white-vertex.glsl").into(),
            stage: wgpu::naga::ShaderStage::Vertex,
            defines: Default::default(),
        },
    });

    let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Glsl {
            shader: include_str!("shaders/video.glsl").into(),
            stage: wgpu::naga::ShaderStage::Fragment,
            defines: Default::default(),
        },
    });

    // TODO(hack3rmann): do a fullscreen triangle instead of quad
    // TODO(hacl3rmann): or use compute pipeline instead
    let triangles = [
        [[-1.0_f32, -1.0], [1.0, 1.0], [-1.0, 1.0]],
        [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0]],
    ];
    let vertex_size = mem::size_of_val(&triangles[0][0]);
    let n_vertices = triangles.len() * triangles[0].len();

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::bytes_of(&triangles),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let video_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("video"),
        size: wgpu::Extent3d {
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    let video_texture_view = video_texture.create_view(&Default::default());
    let video_texture_sampler = device.create_sampler(&Default::default());

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("waywe-bind-group-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("waywe-bind-group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&video_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&video_texture_sampler),
            },
        ],
    });

    #[repr(C)]
    #[derive(Clone, Copy, Default, PartialEq, Debug, Pod, Zeroable)]
    struct PushConst {
        resolution: Vec2,
        time: f32,
    }

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[wgpu::PushConstantRange {
            stages: wgpu::ShaderStages::FRAGMENT,
            range: 0..mem::size_of::<PushConst>() as u32,
        }],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vertex_shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &Default::default(),
                zero_initialize_workgroup_memory: false,
            },
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: vertex_size as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                }],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &fragment_shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &Default::default(),
                zero_initialize_workgroup_memory: false,
            },
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    const FILE_NAMES: &[&CStr] = &[
        c"/home/hack3rmann/Downloads/Telegram Desktop/845514446/video.mp4",
        c"/home/hack3rmann/Videos/ObsRecordings/2025-05-16 15-51-50.mp4",
        c"/home/hack3rmann/Pictures/Wallpapers/night-sky-purple-moon-clouds-3840x2160.mp4",
        c"/home/hack3rmann/Downloads/sample-1.avi",
    ];

    let mut format_context = FormatContext::from_input(FILE_NAMES[0])?;

    let best_stream = format_context.find_best_stream(MediaType::Video)?;
    let best_stream_index = best_stream.index();
    let codec_parameters = best_stream.codec_parameters();
    let frame_rate = codec_parameters.frame_rate();
    let video_size = codec_parameters.video_size().unwrap();
    let Some(video::AudioVideoFormat::Video(video_format)) = codec_parameters.format() else {
        unreachable!();
    };
    let mut codec_context = CodecContext::from_parameters(codec_parameters)?;

    let Some(decoder) = Codec::find_for_id(codec_context.codec_id()) else {
        panic!("failed to find decoder");
    };

    codec_context.open(decoder)?;

    let frame_loop_start = Instant::now();
    let mut last_instant = frame_loop_start;

    let mut packet = Packet::new();
    let mut frame = Frame::new();
    let mut scaled_frame = Frame::new();

    let mut scaler = Scaler::new(
        ScalerFormat {
            size: video_size,
            format: video_format,
        },
        ScalerFormat {
            size: UVec2::new(SCREEN_WIDTH, SCREEN_HEIGHT),
            format: VideoPixelFormat::Rgba8,
        },
        ScalerFlags::BILINEAR,
    )?;

    loop {
        match format_context.read_packet(&mut packet) {
            Ok(()) => {}
            Err(BackendError::EOF) => break,
            error @ Err(..) => error?,
        }

        if packet.stream_index() != best_stream_index {
            continue;
        }

        codec_context.send_packet(&packet)?;

        // TODO(hack3rmann): support for variable frame rate
        let target_frame_time_secs = if !frame_rate.is_zero() {
            frame_rate.inv().to_f32()
        } else {
            RatioI32::new(1, 60).to_f32()
        };

        let target_frame_time = Duration::from_secs_f32(target_frame_time_secs);

        while codec_context.receive_frame(&mut frame).is_ok() {
            let frame_start = Instant::now();
            let time_from_start = last_instant.duration_since(frame_loop_start);
            last_instant = frame_start;
            let time_seconds = time_from_start.as_secs_f32();

            scaler.run(&frame, &mut scaled_frame)?;

            let frame_data = unsafe { scaled_frame.data() };

            gpu_queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &video_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                frame_data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(video_texture.width() * 4),
                    rows_per_image: None,
                },
                video_texture.size(),
            );
            gpu_queue.submit([]);

            let surface_texture = wgpu_surface.get_current_texture().unwrap();
            let surface_view = surface_texture.texture.create_view(&Default::default());

            let mut encoder = device.create_command_encoder(&Default::default());

            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &surface_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                pass.set_pipeline(&pipeline);
                pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                pass.set_push_constants(
                    ShaderStages::FRAGMENT,
                    0,
                    bytemuck::bytes_of(&PushConst {
                        time: time_seconds,
                        resolution: Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32),
                    }),
                );
                pass.set_bind_group(0, &bind_group, &[]);
                pass.draw(0..n_vertices as u32, 0..1);
            }

            _ = gpu_queue.submit([encoder.finish()]);

            surface_texture.present();

            display.roundtrip(queue.as_mut(), client_state.as_ref());

            let render_time = Instant::now().duration_since(frame_start);
            let sleep_time = target_frame_time.saturating_sub(render_time);

            if !sleep_time.is_zero() {
                thread::sleep(sleep_time);
            } else {
                tracing::warn!(?render_time, "frame took too long to prepare");
            }
        }
    }

    Ok(())
}
