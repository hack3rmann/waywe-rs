use crate::{
    effects::{Effect, Effects},
    gpu::Wgpu,
    wayland::MonitorId,
};
use smallvec::SmallVec;
use static_assertions::assert_obj_safe;
use std::ops::Deref;
use waywe_ipc::config::Effects as BuiltinEffects;

#[derive(Default)]
pub struct EffectsBuilder {
    pub configs: SmallVec<[DynEffectConfig; 2]>,
    pub monitor_id: MonitorId,
}

impl EffectsBuilder {
    pub const fn new(monitor_id: MonitorId) -> Self {
        Self {
            configs: SmallVec::new_const(),
            monitor_id,
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

    pub fn build(&self, gpu: &Wgpu) -> Effects {
        Effects(
            self.configs
                .iter()
                .map(|config| config.build_effect(gpu, self.monitor_id))
                .collect(),
        )
    }
}

pub trait EffectConfig: Send + Sync + 'static {
    fn build_effect(&self, gpu: &Wgpu, monitor_id: MonitorId) -> Box<dyn Effect>;
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
