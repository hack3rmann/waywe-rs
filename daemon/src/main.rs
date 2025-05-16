use raw_window_handle::{HasDisplayHandle as _, RawWindowHandle, WaylandWindowHandle};
use std::{error::Error, ffi::CStr, mem, pin::pin};
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
use wgpu::util::DeviceExt as _;

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
        flags: wgpu::InstanceFlags::DEBUG | wgpu::InstanceFlags::VALIDATION,
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
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&wgpu_surface),
        })
        .await
        .expect("failed to request adapter");

    let (device, gpu_queue) = adapter
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
        .unwrap();

    wgpu_surface.configure(
        &device,
        &wgpu_surface
            .get_default_config(&adapter, 2520, 1680)
            .unwrap(),
    );

    let surface_format = wgpu_surface.get_capabilities(&adapter).formats[0];

    let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Glsl {
            shader: include_str!("../tests/shaders/white-vertex.glsl").into(),
            stage: wgpu::naga::ShaderStage::Vertex,
            defines: Default::default(),
        },
    });

    let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Glsl {
            shader: include_str!("../tests/shaders/white-fragment.glsl").into(),
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

    loop {
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

            pass.set_pipeline(&pipeline);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.draw(0..triangle.len() as u32, 0..1);
        }

        _ = gpu_queue.submit([encoder.finish()]);

        surface_texture.present();

        display.roundtrip(queue.as_mut(), client_state.as_ref());
    }
}
