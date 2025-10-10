pub mod blur;

use crate::{
    Monitor,
    effects::blur::Convolve,
    mesh::{CommandEncoder, SurfaceView},
    plugin::Plugin,
    prelude::Wallpaper,
    render::{Render, RenderGpu, RenderSet},
};
use bevy_ecs::prelude::*;
use derive_more::{Deref, DerefMut};
use smallvec::SmallVec;
use static_assertions::assert_obj_safe;
use std::ops::Deref;
use waywe_runtime::{gpu::Wgpu, wayland::MonitorId};

pub const EFFECTS_TEXTURE_USAGES: wgpu::TextureUsages = wgpu::TextureUsages::from_bits(
    wgpu::TextureUsages::COPY_SRC.bits()
        | wgpu::TextureUsages::COPY_DST.bits()
        | wgpu::TextureUsages::STORAGE_BINDING.bits()
        | wgpu::TextureUsages::RENDER_ATTACHMENT.bits(),
)
.unwrap();

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, wallpaper: &mut Wallpaper) {
        let mut effects = Effects::default();

        {
            let gpu = wallpaper.render.resource::<RenderGpu>();
            let monitor_id = wallpaper.render.resource::<Monitor>().id;
            let data = [[1.0; 3]; 3];

            effects.add(Convolve::new(gpu, monitor_id, data.as_flattened()));
        }

        wallpaper
            .render
            .insert_resource(effects)
            .init_resource::<EffectView>()
            .add_systems(
                Render,
                (
                    update_effect_view.in_set(RenderSet::Update),
                    render_effects.in_set(RenderSet::ApplyEffects),
                ),
            );
    }
}

#[derive(Resource)]
pub struct EffectView(pub Option<wgpu::TextureView>);

impl EffectView {
    pub fn new(gpu: &Wgpu, monitor_id: MonitorId) -> Self {
        let (size, format) = {
            let surfaces = gpu.surfaces.read().unwrap();
            let surface = &surfaces[&monitor_id];

            (
                wgpu::Extent3d {
                    width: surface.config.width,
                    height: surface.config.height,
                    depth_or_array_layers: 1,
                },
                surface.format.remove_srgb_suffix(),
            )
        };

        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("effect"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: EFFECTS_TEXTURE_USAGES,
            view_formats: &[],
        });

        let view = texture.create_view(&Default::default());

        Self(Some(view))
    }
}

impl Deref for EffectView {
    type Target = wgpu::TextureView;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl FromWorld for EffectView {
    fn from_world(world: &mut World) -> Self {
        let gpu = world.resource::<RenderGpu>();
        let monitor_id = world.resource::<Monitor>().id;
        Self::new(gpu, monitor_id)
    }
}

pub fn update_effect_view(
    effects: Res<Effects>,
    mut effects_view: ResMut<EffectView>,
    gpu: Res<RenderGpu>,
    monitor: Res<Monitor>,
) {
    if effects.is_empty() && effects_view.0.is_some() {
        effects_view.0 = None;
    } else if !effects.is_empty() && effects_view.0.is_none() {
        *effects_view = EffectView::new(&gpu, monitor.id);
    }
}

pub fn render_effects(
    mut effects: ResMut<Effects>,
    views: ResMut<SurfaceView>,
    effect_view: Res<EffectView>,
    gpu: Res<RenderGpu>,
    mut encoder: ResMut<CommandEncoder>,
) {
    let mut prev_output = None;

    for effect in effects.iter_mut().map(Box::as_mut) {
        let view = prev_output.as_ref().unwrap_or(&**effect_view);

        if let AppliedEffect::WithOutput(texture_view) = effect.apply(&gpu, &mut encoder, view) {
            prev_output = Some(texture_view);
        }
    }

    let result_view = prev_output.as_ref().unwrap_or(&**effect_view);

    encoder.copy_texture_to_texture(
        result_view.texture().as_image_copy(),
        views.texture().as_image_copy(),
        result_view.texture().size(),
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
        surface: &wgpu::TextureView,
    ) -> AppliedEffect;
}
assert_obj_safe!(Effect);

#[derive(Clone, Debug)]
pub enum AppliedEffect {
    Inplace,
    WithOutput(wgpu::TextureView),
}
