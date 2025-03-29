use raw_window_handle::{HasDisplayHandle as _, RawWindowHandle, WaylandWindowHandle};
use std::{
    ffi::CStr,
    mem,
    pin::pin,
    sync::atomic::{AtomicBool, AtomicU32, Ordering::*},
    time::{Duration, Instant},
};
use wayland_client::{
    assert_dispatch_is_empty, interface::{
        Event, WlCompositorCreateSurfaceRequest, WlSurfaceCommitRequest,
        XdgSurfaceAckConfigureRequest, XdgSurfaceConfigureEvent, XdgSurfaceGetToplevelRequest,
        XdgToplevelCloseEvent, XdgToplevelConfigureEvent, XdgToplevelSetAppIdRequest,
        XdgToplevelSetTitleRequest, XdgWmBaseGetXdgSurfaceRequest, XdgWmBasePingEvent,
        XdgWmBasePongRequest,
    }, sys::{
        object::{dispatch::State, FromProxy},
        wire::WlMessage,
    }, Dispatch, HasObjectType, SmallVecMessageBuffer, StackMessageBuffer, WlDisplay, WlObject, WlObjectHandle, WlObjectStorage, WlObjectType, WlProxy, WlRegistry
};
use wgpu::util::DeviceExt as _;

pub const APP_NAME: &CStr = c"simple_wayland_client";
pub const TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug, Default)]
pub struct ClientState {
    pub should_resize: AtomicBool,
    pub ready_to_resize: AtomicBool,
    pub should_close: AtomicBool,
    pub next_width: AtomicU32,
    pub next_height: AtomicU32,
}

impl State for ClientState {}

pub struct WlCompositor;

impl Dispatch for WlCompositor {
    type State = ClientState;

    const ALLOW_EMPTY_DISPATCH: bool = true;

    fn dispatch(
        &mut self,
        _state: &Self::State,
        _storage: &mut WlObjectStorage<'_, Self::State>,
        _message: WlMessage<'_>,
    ) {
        unreachable!()
    }
}

assert_dispatch_is_empty!(WlCompositor);

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
        _state: &Self::State,
        storage: &mut WlObjectStorage<'_, Self::State>,
        message: WlMessage<'_>,
    ) {
        let Some(XdgWmBasePingEvent { serial }) = message.as_event() else {
            return;
        };

        let mut buf = SmallVecMessageBuffer::<1>::new();

        self.handle
            .request(&mut buf, storage, XdgWmBasePongRequest { serial });
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

    const ALLOW_EMPTY_DISPATCH: bool = true;

    fn dispatch(
        &mut self,
        _state: &Self::State,
        _storage: &mut WlObjectStorage<'_, Self::State>,
        _message: WlMessage<'_>,
    ) {
        unreachable!()
    }
}

assert_dispatch_is_empty!(WlSurface);

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
        state: &Self::State,
        storage: &mut WlObjectStorage<'_, Self::State>,
        message: WlMessage<'_>,
    ) {
        let Some(XdgSurfaceConfigureEvent { serial }) = message.as_event() else {
            return;
        };

        let mut buf = SmallVecMessageBuffer::<1>::new();

        self.handle
            .request(&mut buf, storage, XdgSurfaceAckConfigureRequest { serial });

        if state.should_resize.load(Relaxed) {
            state.ready_to_resize.store(true, Relaxed);
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
        state: &Self::State,
        _storage: &mut WlObjectStorage<'_, Self::State>,
        message: WlMessage<'_>,
    ) {
        match message.opcode {
            XdgToplevelConfigureEvent::CODE => {
                let XdgToplevelConfigureEvent { width, height, .. } = message.as_event().unwrap();

                if width != 0 && height != 0 {
                    state.should_resize.store(true, Relaxed);
                    state.next_width.store(width as u32, Relaxed);
                    state.next_height.store(height as u32, Relaxed);
                }
            }
            XdgToplevelCloseEvent::CODE => {
                state.should_close.store(true, Release);
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
    let mut client_state = pin!(ClientState::default());

    let display = WlDisplay::connect(client_state.as_mut()).unwrap();

    let mut buf = StackMessageBuffer::new();
    let mut queue = pin!(display.take_main_queue().unwrap());

    let registry = display.create_registry(&mut buf, queue.as_mut().storage_mut());

    display.roundtrip(queue.as_mut(), client_state.as_ref());

    let compositor =
        WlRegistry::bind::<WlCompositor>(&mut buf, queue.as_mut().storage_mut(), registry).unwrap();

    let wm_base =
        WlRegistry::bind::<WlWmBase>(&mut buf, queue.as_mut().storage_mut(), registry).unwrap();

    display.roundtrip(queue.as_mut(), client_state.as_ref());

    let surface: WlObjectHandle<WlSurface> = compositor.create_object(
        &mut buf,
        queue.as_mut().storage_mut(),
        WlCompositorCreateSurfaceRequest,
    );

    let xdg_surface: WlObjectHandle<WlXdgSurface> = wm_base.create_object(
        &mut buf,
        queue.as_mut().storage_mut(),
        XdgWmBaseGetXdgSurfaceRequest {
            surface: surface.id(),
        },
    );

    let toplevel: WlObjectHandle<WlToplevel> = xdg_surface.create_object(
        &mut buf,
        queue.as_mut().storage_mut(),
        XdgSurfaceGetToplevelRequest,
    );

    toplevel.request(
        &mut buf,
        &queue.as_ref().storage(),
        XdgToplevelSetTitleRequest { title: APP_NAME },
    );
    toplevel.request(
        &mut buf,
        &queue.as_ref().storage(),
        XdgToplevelSetAppIdRequest { app_id: APP_NAME },
    );

    surface.request(&mut buf, &queue.as_ref().storage(), WlSurfaceCommitRequest);
    display.roundtrip(queue.as_mut(), client_state.as_ref());
    surface.request(&mut buf, &queue.as_ref().storage(), WlSurfaceCommitRequest);

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        flags: wgpu::InstanceFlags::DEBUG | wgpu::InstanceFlags::VALIDATION,
        ..Default::default()
    });

    let raw_display_handle = display.display_handle().unwrap().as_raw();
    let raw_window_handle =
        get_window_handle_from_surface(queue.as_ref().storage().object(surface));

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

    let (device, wgpu_queue) = runtime.block_on(async {
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

    let initial_width = client_state.next_width.load(Relaxed);
    let initial_height = client_state.next_height.load(Relaxed);

    assert_ne!(initial_width, 0);
    assert_ne!(initial_height, 0);

    wgpu_surface.configure(
        &device,
        &wgpu_surface
            .get_default_config(&adapter, initial_width, initial_height)
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
        if client_state.should_close.load(Relaxed)
            || Instant::now().duration_since(event_loop_start) >= TIMEOUT
        {
            client_state.should_close.store(false, Relaxed);
            break;
        }

        if client_state.should_resize.load(Relaxed) && client_state.ready_to_resize.load(Relaxed) {
            client_state.should_resize.store(false, Relaxed);
            client_state.ready_to_resize.store(false, Relaxed);

            wgpu_surface.configure(
                &device,
                &wgpu_surface
                    .get_default_config(
                        &adapter,
                        client_state.next_width.load(Relaxed),
                        client_state.next_height.load(Relaxed),
                    )
                    .unwrap(),
            );

            surface.request(&mut buf, &queue.as_ref().storage(), WlSurfaceCommitRequest);

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

        _ = wgpu_queue.submit([encoder.finish()]);

        surface_texture.present();

        display.roundtrip(queue.as_mut(), client_state.as_ref());
    }
}

#[allow(unused)]
#[deprecated = "unsound behavior"]
unsafe fn wait_for_segv() {
    let mut set = unsafe { mem::zeroed() };
    let mut sig_index = 0;

    assert_ne!(-1, unsafe { libc::sigemptyset(&raw mut set) });
    assert_ne!(-1, unsafe { libc::sigaddset(&raw mut set, libc::SIGSEGV) });
    assert_ne!(-1, unsafe {
        libc::sigwait(&raw const set, &raw mut sig_index)
    });

    panic!("caught segfault");
}

#[test]
fn multithread_client() {
    let mut client_state = pin!(ClientState::default());
    let mut buf = StackMessageBuffer::new();

    let display = WlDisplay::connect(client_state.as_mut()).unwrap();

    let mut main_queue = pin!(display.take_main_queue().unwrap());
    let mut side_queue = pin!(display.create_queue().unwrap());

    let registry = display.create_registry(&mut buf, main_queue.as_mut().storage_mut());

    display.roundtrip(main_queue.as_mut(), client_state.as_ref());

    let compositor =
        WlRegistry::bind::<WlCompositor>(&mut buf, main_queue.as_mut().storage_mut(), registry)
            .unwrap();

    main_queue
        .as_mut()
        .storage_mut()
        .move_object(side_queue.as_mut().storage_mut(), compositor)
        .unwrap();

    let wm_base =
        WlRegistry::bind::<WlWmBase>(&mut buf, main_queue.as_mut().storage_mut(), registry)
            .unwrap();

    main_queue
        .as_mut()
        .storage_mut()
        .move_object(side_queue.as_mut().storage_mut(), wm_base)
        .unwrap();

    std::thread::scope(|scope| {
        let display = &display;

        scope.spawn(move || {
            display.roundtrip(side_queue.as_mut(), client_state.as_ref());

            let surface: WlObjectHandle<WlSurface> = compositor.create_object(
                &mut buf,
                side_queue.as_mut().storage_mut(),
                WlCompositorCreateSurfaceRequest,
            );

            let xdg_surface: WlObjectHandle<WlXdgSurface> = wm_base.create_object(
                &mut buf,
                side_queue.as_mut().storage_mut(),
                XdgWmBaseGetXdgSurfaceRequest {
                    surface: surface.id(),
                },
            );

            let toplevel: WlObjectHandle<WlToplevel> = xdg_surface.create_object(
                &mut buf,
                side_queue.as_mut().storage_mut(),
                XdgSurfaceGetToplevelRequest,
            );

            toplevel.request(
                &mut buf,
                &side_queue.as_ref().storage(),
                XdgToplevelSetTitleRequest { title: APP_NAME },
            );

            toplevel.request(
                &mut buf,
                &side_queue.as_ref().storage(),
                XdgToplevelSetAppIdRequest { app_id: APP_NAME },
            );

            surface.request(
                &mut buf,
                &side_queue.as_ref().storage(),
                WlSurfaceCommitRequest,
            );

            display.roundtrip(side_queue.as_mut(), client_state.as_ref());
        });
    });
}
