use bytemuck::{Pod, Zeroable};
use glam::{UVec2, Vec2};
use std::{
    mem::{self, MaybeUninit},
    panic,
    time::Instant,
};
use waywe_rendering_api::{
    VecExt,
    api::{OpaqueRenderer, OpaqueRendererDesc, RenderSurfaceFd, Renderer},
    ffi::PanicPayload,
    import_fd_as_texture,
};
use waywe_runtime::{WallpaperConfig, frame::FrameInfo, shaders::ShaderDescriptor};
use wgpu::util::DeviceExt;

fn create_opaque_renderer(desc: &OpaqueRendererDesc) -> OpaqueRenderer {
    OpaqueRenderer::new(ShaderToyRenderer::new(desc.config))
}

#[unsafe(no_mangle)]
pub extern "C" fn waywe_ffi_create_opaque_renderer(
    desc: &OpaqueRendererDesc,
    out_renderer: &mut MaybeUninit<OpaqueRenderer>,
) -> PanicPayload {
    let result = panic::catch_unwind(move || create_opaque_renderer(desc));

    PanicPayload::map(result, |renderer| {
        out_renderer.write(renderer);
    })
}

#[derive(Debug, Clone)]
pub struct Gpu {
    pub adapter: wgpu::Adapter,
    pub instance: wgpu::Instance,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl Gpu {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            flags: wgpu::InstanceFlags::empty(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::from_env_or_default(),
            display: None,
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .expect("failed to request adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_features: wgpu::Features::IMMEDIATES,
                label: None,
                required_limits: adapter.limits(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        Self {
            adapter,
            instance,
            device,
            queue,
        }
    }
}

pub struct ShaderToyRenderer {
    gpu: Gpu,
    surfaces: Vec<wgpu::Texture>,
    submissions: Vec<Option<wgpu::SubmissionIndex>>,
    wallpaper: ShaderWallpaper,
}

impl ShaderToyRenderer {
    pub fn new(config: WallpaperConfig) -> Self {
        let gpu = pollster::block_on(Gpu::new());

        Self {
            wallpaper: ShaderWallpaper::new(&gpu, config),
            gpu,
            surfaces: vec![],
            submissions: vec![],
        }
    }
}

impl Renderer for ShaderToyRenderer {
    fn render(&mut self) -> FrameInfo {
        let Some(surface) = self.surfaces.first() else {
            panic!("no surface is set");
        };

        if let Some(index) = self.submissions.first_mut().and_then(Option::take) {
            self.gpu
                .device
                .poll(wgpu::PollType::Wait {
                    submission_index: Some(index),
                    timeout: None,
                })
                .unwrap();
        }

        let surface_view = surface.create_view(&Default::default());

        let mut encoder = self.gpu.device.create_command_encoder(&Default::default());

        self.wallpaper.frame(&surface_view, &mut encoder);

        let index = self.gpu.queue.submit([encoder.finish()]);
        self.submissions[0] = Some(index);

        FrameInfo::new_60_fps()
    }

    fn set_surface(&mut self, surface: RenderSurfaceFd, index: u32) {
        let surface = unsafe {
            import_fd_as_texture(
                &self.gpu.device,
                &self.gpu.adapter,
                surface.fd,
                surface.desc,
            )
        };

        let config = WallpaperConfig {
            surface_size: UVec2::new(surface.width(), surface.height()),
            surface_format: surface.format(),
        };

        self.surfaces.set_or_push(index as usize, surface);
        self.submissions.set_or_push(index as usize, None);

        self.wallpaper.configure(&self.gpu, config);
    }

    fn cycle_buffers(&mut self) {
        self.surfaces.rotate_left(0);
        self.submissions.rotate_left(0);
    }
}

#[derive(ShaderDescriptor)]
#[shader(
    path = "crates/examples/shadertoy-computer-were-made-for-cubes/src/shaders/fullscreen-vertex.glsl",
    stage = "vertex",
    label = "computers-were-made-for-cubes"
)]
pub struct FullscreenVertex;

#[derive(ShaderDescriptor)]
#[shader(
    path = "crates/examples/shadertoy-computer-were-made-for-cubes/src/shaders/computers-were-made-for-cubes.glsl",
    stage = "fragment",
    label = "computers-were-made-for-cubes"
)]
pub struct FragmentShader;

#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Debug, Pod, Zeroable)]
pub struct PushConst {
    pub resolution: Vec2,
    pub time: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: Vec2,
}

impl Vertex {
    pub const fn new(x: f32, y: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
        }
    }
}

pub const SCREEN_QUAD: [Vertex; 6] = [
    Vertex::new(-1.0, -1.0),
    Vertex::new(1.0, -1.0),
    Vertex::new(1.0, 1.0),
    Vertex::new(-1.0, -1.0),
    Vertex::new(1.0, 1.0),
    Vertex::new(-1.0, 1.0),
];

pub struct ShaderWallpaper {
    pub start: Option<Instant>,
    pub vertex_shader: wgpu::ShaderModule,
    pub fragment_shader: wgpu::ShaderModule,
    pub vertices: wgpu::Buffer,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub pipeline: wgpu::RenderPipeline,
    pub config: WallpaperConfig,
}

impl ShaderWallpaper {
    pub fn new(gpu: &Gpu, config: WallpaperConfig) -> Self {
        let vertex_shader = gpu
            .device
            .create_shader_module(FullscreenVertex::shader_descriptor());
        let fragment_shader = gpu
            .device
            .create_shader_module(FragmentShader::shader_descriptor());

        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("computer-were-made-for-cubes-pipeline-layout"),
                bind_group_layouts: &[],
                immediate_size: mem::size_of::<PushConst>() as u32,
            });

        let vertices = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("computer-were-made-for-cubes-vertices"),
                contents: bytemuck::cast_slice(&SCREEN_QUAD),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let pipeline = Self::create_pipeline(
            gpu,
            &pipeline_layout,
            &vertex_shader,
            &fragment_shader,
            config.surface_format,
        );

        Self {
            start: None,
            vertex_shader,
            fragment_shader,
            pipeline_layout,
            pipeline,
            config,
            vertices,
        }
    }

    fn create_pipeline(
        gpu: &Gpu,
        layout: &wgpu::PipelineLayout,
        vertex_shader: &wgpu::ShaderModule,
        fragment_shader: &wgpu::ShaderModule,
        surface_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        gpu.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("computer-were-made-for-cubes-pipeline"),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: vertex_shader,
                    entry_point: Some("main"),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &[],
                        zero_initialize_workgroup_memory: false,
                    },
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: mem::size_of::<Vertex>() as u64,
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
                        constants: &[],
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
                multiview_mask: None,
                cache: None,
            })
    }

    pub fn frame(&mut self, surface: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let start = *self.start.get_or_insert_with(Instant::now);

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("computer-were-made-for-cubes-render-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        let resolution = Vec2::new(
            self.config.surface_size.x as f32,
            self.config.surface_size.y as f32,
        );

        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertices.slice(..));
        pass.set_immediates(
            0,
            bytemuck::bytes_of(&PushConst {
                resolution,
                time: start.elapsed().as_secs_f32(),
            }),
        );

        pass.draw(0..SCREEN_QUAD.len() as u32, 0..1);
    }

    pub fn configure(&mut self, gpu: &Gpu, config: WallpaperConfig) {
        if self.config.surface_format != config.surface_format {
            self.pipeline = Self::create_pipeline(
                gpu,
                &self.pipeline_layout,
                &self.vertex_shader,
                &self.fragment_shader,
                config.surface_format,
            );
        }

        self.config = config;
    }
}
