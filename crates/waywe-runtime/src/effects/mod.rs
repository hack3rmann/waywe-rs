pub mod config;
pub mod convolve;

use crate::gpu::Wgpu;
use derive_more::{Deref, DerefMut};
use smallvec::SmallVec;
use static_assertions::assert_obj_safe;

pub const EFFECTS_TEXTURE_USAGES: wgpu::TextureUsages = wgpu::TextureUsages::from_bits(
    wgpu::TextureUsages::COPY_SRC.bits()
        | wgpu::TextureUsages::COPY_DST.bits()
        | wgpu::TextureUsages::STORAGE_BINDING.bits()
        | wgpu::TextureUsages::RENDER_ATTACHMENT.bits()
        | wgpu::TextureUsages::TEXTURE_BINDING.bits(),
)
.unwrap();

pub const EFFECTS_TEXTURE_DESC: wgpu::TextureDescriptor<'static> = wgpu::TextureDescriptor {
    label: Some("effects"),
    // should be replaced
    size: wgpu::Extent3d {
        width: 0,
        height: 0,
        depth_or_array_layers: 0,
    },
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    // should be replaced
    format: wgpu::TextureFormat::Bgra8Unorm,
    usage: EFFECTS_TEXTURE_USAGES,
    view_formats: &[],
};

#[derive(Default, DerefMut, Deref)]
pub struct Effects(pub SmallVec<[Box<dyn Effect>; 2]>);

impl Effects {
    pub const fn new() -> Self {
        Self(SmallVec::new_const())
    }

    pub fn add(&mut self, effect: impl Effect) {
        self.push(Box::new(effect));
    }

    pub fn render(
        &mut self,
        gpu: &Wgpu,
        surface: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let surface_view = surface.texture().create_view(&wgpu::TextureViewDescriptor {
            format: Some(surface.texture().format().remove_srgb_suffix()),
            ..Default::default()
        });

        let mut prev_output = surface_view;
        let mut do_copy = false;

        for effect in self.iter_mut().map(Box::as_mut) {
            if let AppliedEffect::WithOutput(next) = effect.apply(gpu, encoder, &prev_output) {
                prev_output = next;
                do_copy = true;
            }
        }

        if do_copy {
            encoder.copy_texture_to_texture(
                prev_output.texture().as_image_copy(),
                surface.texture().as_image_copy(),
                prev_output.texture().size(),
            );
        }
    }
}

pub trait Effect: Send + Sync + 'static {
    fn apply(
        &mut self,
        gpu: &Wgpu,
        encoder: &mut wgpu::CommandEncoder,
        input: &wgpu::TextureView,
    ) -> AppliedEffect;
}
assert_obj_safe!(Effect);

#[derive(Clone, Debug)]
pub enum AppliedEffect {
    Inplace,
    WithOutput(wgpu::TextureView),
}
