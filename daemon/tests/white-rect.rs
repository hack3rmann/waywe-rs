use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::pin::pin;
use std::time::Instant;
use std::{mem, time::Duration};
use wayland_client::{
    WlObjectHandle,
    interface::WlCompositorCreateSurfaceRequest,
    sys::{
        display::WlDisplay,
        object::{
            default_impl::{Compositor, Surface},
            dispatch::NoState,
            registry::WlRegistry,
        },
        wire::SmallVecMessageBuffer,
    },
};
use wgpu::util::DeviceExt as _;

pub const TIMEOUT: Duration = Duration::from_millis(500);

#[tokio::test]
async fn use_wgpu_to_draw_anything() {
    let mut buf = SmallVecMessageBuffer::<8>::new();

    let mut state = pin!(NoState);
    let display = WlDisplay::connect(state.as_mut()).unwrap();
    let mut queue = pin!(display.take_main_queue().unwrap());
    let registry = display.create_registry(&mut buf, queue.as_mut().storage_mut());

    display.roundtrip(queue.as_mut(), state.as_ref());

    let compositor =
        WlRegistry::bind::<Compositor>(&mut buf, queue.as_mut().storage_mut(), registry).unwrap();

    let surface: WlObjectHandle<Surface> = compositor.create_object(
        &mut buf,
        queue.as_mut().storage_mut(),
        WlCompositorCreateSurfaceRequest,
    );

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        flags: wgpu::InstanceFlags::DEBUG | wgpu::InstanceFlags::VALIDATION,
        ..Default::default()
    });

    let raw_display_handle = display.display_handle().unwrap().as_raw();
    let raw_window_handle = queue
        .as_ref()
        .storage()
        .object(surface)
        .window_handle()
        .unwrap()
        .as_raw();

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

    let (device, wgpu_queue) = adapter
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

    const WIDTH: u32 = 2520;
    const HEIGHT: u32 = 1680;
    wgpu_surface.configure(
        &device,
        &wgpu_surface
            .get_default_config(&adapter, WIDTH, HEIGHT)
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
                array_stride: mem::size_of_val(&triangle[0]) as u64,
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

    let event_loop_start = Instant::now();

    loop {
        if Instant::now().duration_since(event_loop_start) >= TIMEOUT {
            break;
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
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
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

        wgpu_queue.submit([encoder.finish()]);

        surface_texture.present();
    }
}
