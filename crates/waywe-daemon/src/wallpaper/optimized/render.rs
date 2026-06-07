use crate::wallpaper::Wallpaper;
use libloading::Library;
use std::{mem::MaybeUninit, path::Path};
use waywe_runtime::{WallpaperConfig, frame::FrameInfo, gpu::Wgpu};
use waywe_subrenderer::{
    FfiTextureDescriptor,
    api::{
        CREATE_OPAQUE_RENDERER_NAME, CreateOpaqueRendererFn, OpaqueRenderer, OpaqueRendererDesc,
        RenderSurfaceFd, Renderer,
    },
    texture_export_fd,
};

pub struct RenderWallpaper {
    // NOTE(hack3rmann): `renderer` must be dropped before `_lib`
    renderer: OpaqueRenderer,
    _lib: Library,
    surface: wgpu::Texture,
    config: WallpaperConfig,
}

unsafe impl Send for RenderWallpaper {}
unsafe impl Sync for RenderWallpaper {}

impl RenderWallpaper {
    pub fn load(path: impl AsRef<Path>, gpu: &Wgpu, config: WallpaperConfig) -> Self {
        let lib = unsafe { Library::new(path.as_ref()) }.unwrap();

        let create_opaque_renderer = unsafe {
            lib.get::<CreateOpaqueRendererFn>(CREATE_OPAQUE_RENDERER_NAME)
                .unwrap()
        };

        let mut renderer = MaybeUninit::uninit();

        let panic = create_opaque_renderer(&OpaqueRendererDesc { config }, &mut renderer);
        panic.propagate_if_any();

        let mut renderer = unsafe { renderer.assume_init() };

        let surface_desc = Self::surface_desc(config);
        let surface = gpu.device.create_texture(&surface_desc);

        renderer.set_surface(RenderSurfaceFd {
            fd: unsafe { texture_export_fd(&gpu.device, &surface) },
            desc: FfiTextureDescriptor::from(surface_desc),
        });

        Self {
            renderer,
            _lib: lib,
            surface,
            config,
        }
    }

    fn surface_desc(config: WallpaperConfig) -> wgpu::TextureDescriptor<'static> {
        wgpu::TextureDescriptor {
            label: Some("waywe-subrenderer-surface"),
            size: wgpu::Extent3d {
                width: config.surface_size.x,
                height: config.surface_size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.surface_format,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        }
    }
}

impl Wallpaper for RenderWallpaper {
    fn configure(&mut self, gpu: &Wgpu, config: WallpaperConfig) {
        if config == self.config {
            return;
        }

        let desc = Self::surface_desc(config);
        self.surface = gpu.device.create_texture(&desc);

        self.renderer.set_surface(RenderSurfaceFd {
            fd: unsafe { texture_export_fd(&gpu.device, &self.surface) },
            desc: FfiTextureDescriptor::from(desc),
        });
    }

    fn frame(
        &mut self,
        _: &Wgpu,
        surface: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) -> FrameInfo {
        let info = self.renderer.render();

        encoder.copy_texture_to_texture(
            self.surface.as_image_copy(),
            surface.texture().as_image_copy(),
            surface.texture().size(),
        );

        info
    }
}
