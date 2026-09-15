use crate::{
    WallpaperConfig,
    effects::{Effect, Effects},
    gpu::Wgpu,
};
use smallvec::SmallVec;
use static_assertions::assert_obj_safe;
use std::ops::Deref;
use waywe_config::Effects as BuiltinEffects;

#[derive(Default)]
pub struct EffectsBuilder {
    pub configs: SmallVec<[DynEffectConfig; 2]>,
}

impl EffectsBuilder {
    pub const fn new() -> Self {
        Self {
            configs: SmallVec::new_const(),
        }
    }

    pub fn add(&mut self, config: impl Into<DynEffectConfig>) -> &mut Self {
        self.configs.push(config.into());
        self
    }

    pub fn add_builtins<'a>(
        &mut self,
        configs: impl IntoIterator<Item = &'a BuiltinEffects>,
    ) -> &mut Self {
        for config in configs {
            match config {
                BuiltinEffects::Convolve(config) => _ = self.add(config.clone()),
                BuiltinEffects::Blur(config) => _ = self.add(*config),
            }
        }

        self
    }

    pub fn build(&self, gpu: &Wgpu, wallpaper_config: WallpaperConfig) -> Effects {
        Effects(
            self.configs
                .iter()
                .map(|config| config.build_effect(gpu, wallpaper_config))
                .collect(),
        )
    }
}

pub trait EffectConfig: Send + Sync + 'static {
    fn build_effect(&self, gpu: &Wgpu, config: WallpaperConfig) -> Box<dyn Effect>;
}
assert_obj_safe!(EffectConfig);

pub struct DynEffectConfig(pub Box<dyn EffectConfig>);

impl Deref for DynEffectConfig {
    type Target = dyn EffectConfig;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<C: EffectConfig> From<C> for DynEffectConfig {
    fn from(value: C) -> Self {
        Self(Box::new(value))
    }
}

impl From<Box<dyn EffectConfig>> for DynEffectConfig {
    fn from(value: Box<dyn EffectConfig>) -> Self {
        Self(value)
    }
}
