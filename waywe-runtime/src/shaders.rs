use std::{
    any::TypeId,
    collections::HashMap,
    marker::PhantomData,
    ops::Deref,
    sync::{RwLock, RwLockReadGuard},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShaderId(pub TypeId);

pub trait ShaderDescriptor: 'static {
    fn shader_descriptor() -> wgpu::ShaderModuleDescriptor<'static>;
}

#[derive(Default)]
pub struct ShaderCache {
    shaders: RwLock<HashMap<TypeId, wgpu::ShaderModule>>,
}

impl ShaderCache {
    pub fn contains<S: ShaderDescriptor>(&self) -> bool {
        let shaders = self.shaders.read().unwrap();
        shaders.contains_key(&TypeId::of::<S>())
    }

    // TODO(hack3rmann): use .get_or_init() instead
    pub fn initialize<S: ShaderDescriptor>(&self, device: &wgpu::Device) {
        if self.contains::<S>() {
            return;
        }

        let shader = device.create_shader_module(S::shader_descriptor());
        let mut map = self.shaders.write().unwrap();
        _ = map.insert(TypeId::of::<S>(), shader);
    }

    pub fn get<S: ShaderDescriptor>(&self) -> Option<RwLockShaderReadGuard<'_, S>> {
        let shaders = self.shaders.read().unwrap();

        if !shaders.contains_key(&TypeId::of::<S>()) {
            return None;
        }

        Some(RwLockShaderReadGuard {
            shaders,
            _p: PhantomData,
        })
    }
}

pub struct RwLockShaderReadGuard<'s, S> {
    shaders: RwLockReadGuard<'s, HashMap<TypeId, wgpu::ShaderModule>>,
    _p: PhantomData<&'s S>,
}

impl<S: 'static> Deref for RwLockShaderReadGuard<'_, S> {
    type Target = wgpu::ShaderModule;

    fn deref(&self) -> &Self::Target {
        &self.shaders[&TypeId::of::<S>()]
    }
}
