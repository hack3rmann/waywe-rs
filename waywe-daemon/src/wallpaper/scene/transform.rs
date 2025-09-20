#![allow(clippy::type_complexity)]

use super::{render::Renderer, wallpaper::WallpaperBetter};
use crate::wallpaper::scene::{
    ScenePlugin, Update, Wallpaper,
    extract::Extract,
    plugin::Plugin,
    render::{EntityMap, RenderPlugin, SceneExtract},
};
use bevy_ecs::prelude::*;
use glam::{Mat4, Quat, Vec3};
use smallvec::SmallVec;

pub struct TransformPlugin;

impl ScenePlugin for TransformPlugin {
    fn init(self, scene: &mut Wallpaper) {
        scene.add_systems(Update, propagate_transforms);
    }
}

impl RenderPlugin for TransformPlugin {
    fn init(self, renderer: &mut Renderer) {
        renderer.add_systems(SceneExtract, extract_transforms);
    }
}

impl Plugin for TransformPlugin {
    fn build(&self, wallpaper: &mut WallpaperBetter) {
        wallpaper.main.add_systems(Update, propagate_transforms);
        wallpaper
            .render
            .add_systems(SceneExtract, extract_transforms);
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Component)]
#[require(GlobalTransform)]
pub struct Transform {
    pub translation: Vec3,
    pub scale: Vec3,
    pub rotation: Quat,
}

impl Transform {
    pub const DEFAULT: Self = Self {
        translation: Vec3::ZERO,
        scale: Vec3::ONE,
        rotation: Quat::IDENTITY,
    };

    pub const fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Self::DEFAULT
        }
    }

    pub const fn with_translation(self, translation: Vec3) -> Self {
        Self {
            translation,
            ..self
        }
    }

    pub fn scaled_by(self, scale: Vec3) -> Self {
        Self {
            scale: self.scale * scale,
            ..self
        }
    }

    pub fn combine(self, other: Self) -> Self {
        Self {
            translation: self.translation + other.translation,
            scale: self.scale * other.scale,
            rotation: self.rotation * other.rotation,
        }
    }

    pub fn to_model(self) -> Mat4 {
        Mat4::from_translation(self.translation)
            * Mat4::from_quat(self.rotation)
            * Mat4::from_scale(self.scale)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Default, PartialEq, Clone, Copy, Component)]
pub struct GlobalTransform(pub Transform);

// TODO: make it faster
pub fn propagate_transforms(
    mut entity_stack: Local<SmallVec<[(Entity, Transform); 16]>>,
    mut commands: Commands,
    roots: Query<(Entity, &Transform), Without<ChildOf>>,
    children: Query<&Children>,
    transforms: Query<&Transform>,
) {
    for (root_id, &root_transform) in &roots {
        entity_stack.clear();

        commands
            .entity(root_id)
            .insert(GlobalTransform(root_transform));

        entity_stack.push((root_id, root_transform));

        while let Some((id, transform)) = entity_stack.pop() {
            for &child_id in children.get(id).map(|c| &**c).unwrap_or(&[]) {
                let child_transform = match transforms.get(child_id) {
                    Ok(&transform) => transform,
                    Err(_) => {
                        let default = Transform::default();
                        commands.entity(child_id).insert(default);
                        default
                    }
                };

                let global_transform = transform.combine(child_transform);

                commands
                    .entity(child_id)
                    .insert(GlobalTransform(global_transform));

                entity_stack.push((child_id, global_transform));
            }
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone, Copy, Component)]
pub struct ModelMatrix(pub Mat4);

pub fn extract_transforms(
    entity_map: Res<EntityMap>,
    mut commands: Commands,
    transforms: Extract<Query<(Entity, &Transform), Changed<Transform>>>,
) {
    for (main_id, &transform) in &transforms {
        let Some(&id) = entity_map.get(&main_id) else {
            continue;
        };

        let Ok(mut entity) = commands.get_entity(id) else {
            continue;
        };

        entity.insert(ModelMatrix(transform.to_model()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::wayland::MonitorId;

    const MONITOR_ID: MonitorId = MonitorId::new(1).unwrap();

    #[test]
    fn simple_propagate() {
        let mut scene = Wallpaper::new(MONITOR_ID);

        scene.add_plugin(TransformPlugin);

        let r21 = scene
            .world
            .spawn(Transform::from_translation(3.0 * Vec3::ONE))
            .id();

        let r1 = scene
            .world
            .spawn(Transform::from_translation(Vec3::ONE))
            .id();

        let r2 = scene
            .world
            .spawn(Transform::from_translation(2.0 * Vec3::ONE))
            .add_child(r21)
            .id();

        let root = scene
            .world
            .spawn(Transform::from_translation(Vec3::ONE))
            .add_children(&[r1, r2])
            .id();

        scene.update();

        let root = scene.world.entity(root).get::<GlobalTransform>().unwrap().0;
        let r2 = scene.world.entity(r2).get::<GlobalTransform>().unwrap().0;
        let r1 = scene.world.entity(r1).get::<GlobalTransform>().unwrap().0;
        let r21 = scene.world.entity(r21).get::<GlobalTransform>().unwrap().0;

        assert_eq!(root.translation, Vec3::ONE);
        assert_eq!(r1.translation, 2.0 * Vec3::ONE);
        assert_eq!(r2.translation, 3.0 * Vec3::ONE);
        assert_eq!(r21.translation, 6.0 * Vec3::ONE);
    }
}
