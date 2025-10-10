use crate::{
    Monitor,
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
        | wgpu::TextureUsages::STORAGE_BINDING.bits()
        | wgpu::TextureUsages::RENDER_ATTACHMENT.bits(),
)
.unwrap();

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, wallpaper: &mut Wallpaper) {
        wallpaper.render.init_resource::<Effects>().add_systems(
            Render,
            (
                alter_surface_view.in_set(RenderSet::Update),
                render_effects.in_set(RenderSet::ApplyEffects),
            ),
        );
    }
}

pub fn alter_surface_view(
    effects: Res<Effects>,
    mut views: ResMut<SurfaceView>,
    gpu: Res<RenderGpu>,
    monitor: Res<Monitor>,
) {
    let requires_surface_storage = effects
        .iter()
        .map(Box::as_ref)
        .any(Effect::requires_surface_storage_usage);

    if !requires_surface_storage && views.effect.is_some() {
        _ = views.effect.take();
    } else if requires_surface_storage && views.effect.is_none() {
        views.init_effect(&gpu, monitor.id);
    }
}

pub fn render_effects(
    mut effects: ResMut<Effects>,
    views: ResMut<SurfaceView>,
    gpu: Res<RenderGpu>,
    mut encoder: ResMut<CommandEncoder>,
) {
    let mut prev_output = None;

    for effect in effects.iter_mut().map(Box::as_mut) {
        let surface = match (
            effect.requires_surface_storage_usage(),
            views.effect.as_ref(),
            prev_output.as_ref(),
        ) {
            (true, Some(surface), None) | (_, _, Some(surface)) => surface,
            (false, _, None) => &views,
            (true, None, None) => {
                panic!("effect requires a special effect texture, but it is not present")
            }
        };

        if let AppliedEffect::WithOutput(texture_view) = effect.apply(&gpu, &mut encoder, surface) {
            prev_output = Some(texture_view);
        }
    }

    if let Some(result) = prev_output {
        encoder.copy_texture_to_texture(
            result.texture().as_image_copy(),
            views.texture().as_image_copy(),
            result.texture().size(),
        );
    }
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
        surface: &wgpu::TextureView,
    ) -> AppliedEffect;

    fn requires_surface_storage_usage(&self) -> bool {
        true
    }
}
assert_obj_safe!(Effect);

#[derive(Clone, Debug)]
pub enum AppliedEffect {
    Inplace,
    WithOutput(wgpu::TextureView),
}
