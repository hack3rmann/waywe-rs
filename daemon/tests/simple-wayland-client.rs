use raw_window_handle::{HasDisplayHandle as _, RawWindowHandle, WaylandWindowHandle};
use std::{
    ffi::CStr,
    mem,
    pin::{Pin, pin},
    time::{Duration, Instant},
};
use wayland::{
    Dispatch, HasObjectType, SmallVecMessageBuffer, StackMessageBuffer, WlDisplay, WlObject,
    WlObjectHandle, WlObjectType, WlProxy, WlRegistry,
    interface::{
        Event, WlCompositorCreateSurfaceRequest, WlSurfaceCommitRequest, WlToplevelCloseEvent,
        WlToplevelConfigureEvent, WlToplevelSetAppIdRequest, WlToplevelSetTitleRequest,
        WlWmBaseGetXdgSurfaceRequest, WlWmBasePingEvent, WlWmBasePongRequest,
        WlXdgSurfaceAckConfigureRequest, WlXdgSurfaceConfigureEvent,
        WlXdgSurfaceGetToplevelRequest,
    },
    sys::{
        object::{FromProxy, dispatch::State},
        wire::WlMessage,
    },
};
use wgpu::util::DeviceExt as _;

pub const APP_NAME: &CStr = c"simple_wayland_client";
pub const TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Clone, Debug, Default)]
pub struct ClientState {
    pub should_resize: bool,
    pub ready_to_resize: bool,
    pub should_close: bool,
    pub next_width: u32,
    pub next_height: u32,
}

impl State for ClientState {}

pub struct WlCompositor;

impl Dispatch for WlCompositor {
    type State = ClientState;
}

impl FromProxy for WlCompositor {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl HasObjectType for WlCompositor {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Compositor;
}

pub struct WlWmBase {
    pub handle: WlObjectHandle<Self>,
}

impl Dispatch for WlWmBase {
    type State = ClientState;

    fn dispatch(
        &mut self,
        _state: Pin<&mut Self::State>,
        storage: Pin<&mut wayland::WlObjectStorage<'_, Self::State>>,
        message: WlMessage<'_>,
    ) {
        let Some(WlWmBasePingEvent { serial }) = message.as_event() else {
            return;
        };

        let mut buf = SmallVecMessageBuffer::<1>::new();

        self.handle
            .request(&mut buf, &storage, WlWmBasePongRequest { serial });
    }
}

impl FromProxy for WlWmBase {
    fn from_proxy(proxy: &WlProxy) -> Self {
        Self {
            handle: WlObjectHandle::new(proxy.id()),
        }
    }
}

impl HasObjectType for WlWmBase {
    const OBJECT_TYPE: WlObjectType = WlObjectType::XdgWmBase;
}

pub struct WlSurface;

impl Dispatch for WlSurface {
    type State = ClientState;
}

impl HasObjectType for WlSurface {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Surface;
}

impl FromProxy for WlSurface {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

pub struct WlXdgSurface {
    pub handle: WlObjectHandle<Self>,
}

impl Dispatch for WlXdgSurface {
    type State = ClientState;

    fn dispatch(
        &mut self,
        mut state: Pin<&mut Self::State>,
        storage: Pin<&mut wayland::WlObjectStorage<'_, Self::State>>,
        message: WlMessage<'_>,
    ) {
        let Some(WlXdgSurfaceConfigureEvent { serial }) = message.as_event() else {
            return;
        };

        let mut buf = SmallVecMessageBuffer::<1>::new();

        self.handle.request(
            &mut buf,
            &storage,
            WlXdgSurfaceAckConfigureRequest { serial },
        );

        if state.should_resize {
            state.ready_to_resize = true;
        }
    }
}

impl HasObjectType for WlXdgSurface {
    const OBJECT_TYPE: WlObjectType = WlObjectType::XdgSurface;
}

impl FromProxy for WlXdgSurface {
    fn from_proxy(proxy: &WlProxy) -> Self {
        Self {
            handle: WlObjectHandle::new(proxy.id()),
        }
    }
}

#[derive(Debug, PartialEq, Default, Clone, Copy, PartialOrd, Eq, Ord, Hash)]
pub struct WlToplevel;

impl Dispatch for WlToplevel {
    type State = ClientState;

    fn dispatch(
        &mut self,
        mut state: Pin<&mut Self::State>,
        _storage: Pin<&mut wayland::WlObjectStorage<'_, Self::State>>,
        message: WlMessage<'_>,
    ) {
        match message.opcode {
            WlToplevelConfigureEvent::CODE => {
                let WlToplevelConfigureEvent { width, height, .. } = message.as_event().unwrap();

                if width != 0 && height != 0 {
                    state.should_resize = true;
                    state.next_width = width;
                    state.next_height = height;
                }
            }
            WlToplevelCloseEvent::CODE => {
                state.should_close = true;
            }
            _ => {}
        }
    }
}

impl HasObjectType for WlToplevel {
    const OBJECT_TYPE: WlObjectType = WlObjectType::XdgToplevel;
}

impl FromProxy for WlToplevel {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

fn get_window_handle_from_surface(surface: &WlObject<WlSurface>) -> RawWindowHandle {
    RawWindowHandle::Wayland(WaylandWindowHandle::new(surface.proxy().as_raw()))
}

pub struct Swapchain {
    pipeline: wgpu::RenderPipeline,
}

impl Swapchain {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        pipeline_layout: &wgpu::PipelineLayout,
        vertex_shader: &wgpu::ShaderModule,
        fragment_shader: &wgpu::ShaderModule,
        vertex_size: u64,
    ) -> Self {
        Self {
            pipeline: device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(pipeline_layout),
                vertex: wgpu::VertexState {
                    module: vertex_shader,
                    entry_point: Some("main"),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &Default::default(),
                        zero_initialize_workgroup_memory: false,
                    },
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: vertex_size,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        }],
                    }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: fragment_shader,
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
            }),
        }
    }
}

#[test]
fn simple_wayland_client() {
    tracing_subscriber::fmt::init();

    let mut client_state = pin!(ClientState::default());

    let display = WlDisplay::connect(client_state.as_mut()).unwrap();

    let mut buf = StackMessageBuffer::new();
    let mut storage = pin!(display.create_storage());

    let registry = display.create_registry(&mut buf, storage.as_mut());

    display.dispatch_all_pending(storage.as_mut(), client_state.as_mut());

    let compositor =
        WlRegistry::bind::<WlCompositor>(&mut buf, storage.as_mut(), registry).unwrap();

    let wm_base = WlRegistry::bind::<WlWmBase>(&mut buf, storage.as_mut(), registry).unwrap();

    display.dispatch_all_pending(storage.as_mut(), client_state.as_mut());

    let surface: WlObjectHandle<WlSurface> =
        compositor.create_object(&mut buf, storage.as_mut(), WlCompositorCreateSurfaceRequest);

    let xdg_surface: WlObjectHandle<WlXdgSurface> = wm_base.create_object(
        &mut buf,
        storage.as_mut(),
        WlWmBaseGetXdgSurfaceRequest {
            surface: surface.id(),
        },
    );

    let toplevel: WlObjectHandle<WlToplevel> =
        xdg_surface.create_object(&mut buf, storage.as_mut(), WlXdgSurfaceGetToplevelRequest);

    toplevel.request(&mut buf, &storage, WlToplevelSetTitleRequest(APP_NAME));
    toplevel.request(&mut buf, &storage, WlToplevelSetAppIdRequest(APP_NAME));

    surface.request(&mut buf, &storage, WlSurfaceCommitRequest);
    display.dispatch_all_pending(storage.as_mut(), client_state.as_mut());
    surface.request(&mut buf, &storage, WlSurfaceCommitRequest);

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        flags: wgpu::InstanceFlags::DEBUG | wgpu::InstanceFlags::VALIDATION,
        ..Default::default()
    });

    let raw_display_handle = display.display_handle().unwrap().as_raw();
    let raw_window_handle = get_window_handle_from_surface(storage.object(surface));

    let wgpu_surface = unsafe {
        instance
            .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle,
                raw_window_handle,
            })
            .unwrap()
    };

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let adapter = runtime.block_on(async {
        instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&wgpu_surface),
            })
            .await
            .expect("failed to request adapter")
    });

    let (device, queue) = runtime.block_on(async {
        adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    label: None,
                    required_limits: adapter.limits(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .unwrap()
    });

    assert_ne!(client_state.next_width, 0);
    assert_ne!(client_state.next_height, 0);

    wgpu_surface.configure(
        &device,
        &wgpu_surface
            .get_default_config(&adapter, client_state.next_width, client_state.next_height)
            .unwrap(),
    );

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
            shader: include_str!("shaders/white-fragment.glsl").into(),
            stage: wgpu::naga::ShaderStage::Fragment,
            defines: Default::default(),
        },
    });

    let triangle: [[f32; 2]; 3] = [[-0.5, -0.5], [0.5, -0.5], [0.0, 0.5]];
    let vertex_size = mem::size_of_val(&triangle[0]);

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::bytes_of(&triangle),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let mut swapchain = Swapchain::new(
        &device,
        surface_format,
        &pipeline_layout,
        &vertex_shader,
        &fragment_shader,
        vertex_size as u64,
    );

    let event_loop_start = Instant::now();

    loop {
        if client_state.should_close || Instant::now().duration_since(event_loop_start) >= TIMEOUT {
            client_state.should_close = false;
            break;
        }

        if client_state.should_resize && client_state.ready_to_resize {
            client_state.should_resize = false;
            client_state.ready_to_resize = false;

            wgpu_surface.configure(
                &device,
                &wgpu_surface
                    .get_default_config(&adapter, client_state.next_width, client_state.next_height)
                    .unwrap(),
            );

            surface.request(&mut buf, &storage, WlSurfaceCommitRequest);

            swapchain = Swapchain::new(
                &device,
                surface_format,
                &pipeline_layout,
                &vertex_shader,
                &fragment_shader,
                vertex_size as u64,
            );
        }

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
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&swapchain.pipeline);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.draw(0..triangle.len() as u32, 0..1);
        }

        _ = queue.submit([encoder.finish()]);

        surface_texture.present();

        display.dispatch_all_pending(storage.as_mut(), client_state.as_mut());
    }
}
