pub mod convolve;

use crate::{
    Monitor,
    effects::convolve::Convolve,
    mesh::{CommandEncoder, SurfaceView},
    plugin::Plugin,
    prelude::Wallpaper,
    render::{Render, RenderGpu, RenderSet},
};
use bevy_ecs::prelude::*;
use derive_more::{Deref, DerefMut};
use smallvec::SmallVec;
use static_assertions::assert_obj_safe;
use waywe_runtime::gpu::Wgpu;

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

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, wallpaper: &mut Wallpaper) {
        let mut effects = Effects::default();

        {
            let gpu = wallpaper.render.resource::<RenderGpu>();
            let monitor_id = wallpaper.render.resource::<Monitor>().id;
            let data = [[1.0 / 9.0; 3]; 3];

            effects.add(Convolve::new(gpu, monitor_id, data.as_flattened()));
        }

        wallpaper
            .render
            .insert_resource(effects)
            .add_systems(Render, render_effects.in_set(RenderSet::ApplyEffects));
    }
}

pub fn render_effects(
    mut effects: ResMut<Effects>,
    surface: ResMut<SurfaceView>,
    gpu: Res<RenderGpu>,
    mut encoder: ResMut<CommandEncoder>,
) {
    let surface_view = surface.texture().create_view(&wgpu::TextureViewDescriptor {
        format: Some(surface.texture().format().remove_srgb_suffix()),
        ..Default::default()
    });

    let mut prev_output = surface_view;

    for effect in effects.iter_mut().map(Box::as_mut) {
        if let AppliedEffect::WithOutput(next) = effect.apply(&gpu, &mut encoder, &prev_output) {
            prev_output = next;
        }
    }

    encoder.copy_texture_to_texture(
        prev_output.texture().as_image_copy(),
        surface.texture().as_image_copy(),
        prev_output.texture().size(),
    );
}

#[derive(Resource, Default, DerefMut, Deref)]
pub struct Effects(pub SmallVec<[Box<dyn Effect>; 2]>);

impl Effects {
    pub fn add(&mut self, effect: impl Effect) {
        self.push(Box::new(effect));
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
