//! Transform components and systems for positioning entities.
//!
//! This module provides components and systems for positioning, scaling,
//! and rotating entities in 2D space.
//!
//! # Components
//!
//! - [`Transform`]: Local transformation (position, scale, rotation)
//! - [`GlobalTransform`]: World-space transformation
//! - [`ModelMatrix`]: Transformation matrix for rendering
//!
//! # Systems
//!
//! - [`propagate_transforms`]: Update world-space transforms from hierarchy
//! - [`propagate_simple_transforms`]: Update simple transforms without hierarchy
//! - [`extract_transforms`]: Extract transforms for rendering

use crate::{
    Update,
    extract::Extract,
    plugin::Plugin,
    render::{EntityMap, SceneExtract},
    wallpaper::Wallpaper,
};
use glam::{Mat4, Quat, Vec3};
use smallvec::SmallVec;
use waywe_ecs::prelude::*;
use waywe_uuid::TypeUuid;

/// Plugin for transform functionality.
///
/// Adds systems for updating and extracting transforms.
pub struct TransformPlugin;

impl Plugin for TransformPlugin {
    fn build(&self, wallpaper: &mut Wallpaper) {
        wallpaper
            .main
            .add_systems(Update, (propagate_transforms, propagate_simple_transforms));
        wallpaper
            .render
            .add_systems(SceneExtract, extract_transforms);
    }
}

/// Local transformation component.
///
/// Defines the position, scale, and rotation of an entity relative to its parent.
#[derive(Debug, PartialEq, Clone, Copy, Component, TypeUuid)]
#[uuid = "311a65a4-0a24-42b4-8519-7ddd68f45a7a"]
#[require(GlobalTransform)]
pub struct Transform {
    /// Translation (position) in 3D space.
    pub translation: Vec3,
    /// Scale factors along each axis.
    pub scale: Vec3,
    /// Rotation as a quaternion.
    pub rotation: Quat,
}

impl Transform {
    /// Default transform (no translation, unit scale, no rotation).
    pub const DEFAULT: Self = Self {
        translation: Vec3::ZERO,
        scale: Vec3::ONE,
        rotation: Quat::IDENTITY,
    };

    /// Create a transform with a specific translation.
    pub const fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Self::DEFAULT
        }
    }

    /// Set the translation of this transform.
    pub const fn with_translation(self, translation: Vec3) -> Self {
        Self {
            translation,
            ..self
        }
    }

    /// Scale this transform by additional factors.
    pub fn scaled_by(self, scale: Vec3) -> Self {
        Self {
            scale: self.scale * scale,
            ..self
        }
    }

    /// Combine this transform with another transform.
    pub fn combine(self, other: Self) -> Self {
        Self {
            translation: self.translation + other.translation,
            scale: self.scale * other.scale,
            rotation: self.rotation * other.rotation,
        }
    }

    /// Convert this transform to a model matrix.
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

/// Global (world-space) transformation component.
///
/// Represents the absolute position, scale, and rotation of an entity
/// in world space, calculated from the hierarchy.
#[derive(Debug, Default, PartialEq, Clone, Copy, Component, TypeUuid)]
#[uuid = "06353292-33c6-4b65-abb6-432d8780f9e5"]
pub struct GlobalTransform(pub Transform);

/// System to update simple transforms (no hierarchy).
///
/// Updates [`GlobalTransform`] components for entities that don't have
/// parents or children.
pub fn propagate_simple_transforms(
    mut transforms: Query<
        (&Transform, &mut GlobalTransform),
        (Changed<Transform>, Without<ChildOf>, Without<Children>),
    >,
) {
    for (&transform, mut global_transform) in &mut transforms {
        *global_transform = GlobalTransform(transform);
    }
}

/// System to propagate transforms through the hierarchy.
///
/// Updates [`GlobalTransform`] components for entities with parents and children,
/// ensuring child transforms are correctly calculated relative to their parents.
pub fn propagate_transforms(
    mut entity_stack: Local<SmallVec<[(Entity, Transform); 16]>>,
    mut commands: Commands,
    roots: Query<(Entity, &Transform), (Without<ChildOf>, With<Children>)>,
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

#[derive(Debug, Default, PartialEq, Clone, Copy, Component, TypeUuid)]
#[uuid = "473c6451-3793-407f-971c-d00a0055d620"]
pub struct PreExtractTransform(pub Transform);

/// Model matrix for rendering.
///
/// A 4x4 transformation matrix that can be directly used in shaders.
#[derive(Debug, Default, PartialEq, Clone, Copy, Component, TypeUuid)]
#[uuid = "992b9dfe-fc85-42ee-8905-7c6de0dcb681"]
pub struct ModelMatrix(pub Mat4);

/// System to extract transforms for rendering.
///
/// Copies transform data from the main world to the render world.
pub fn extract_transforms(
    entity_map: Res<EntityMap>,
    mut commands: Commands,
    transforms: Extract<Query<(Entity, &Transform), Changed<Transform>>>,
    mut models: Query<(&mut ModelMatrix, Option<&PreExtractTransform>)>,
) {
    for (main_id, &transform) in &transforms {
        let Some(&id) = entity_map.get(&main_id) else {
            continue;
        };

        let Ok(mut entity) = commands.get_entity(id) else {
            continue;
        };

        if let Ok((mut model, pre_transform)) = models.get_mut(id) {
            let transform = transform.combine(pre_transform.cloned().unwrap_or_default().0);
            *model = ModelMatrix(transform.to_model());
        } else {
            entity.insert(ModelMatrix(transform.to_model()));
        }
    }
}
