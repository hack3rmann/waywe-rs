//! Rendering system components and schedules.
//!
//! This module provides the core components and schedules for the render world.
//! It handles GPU operations, entity mapping between worlds, and monitor events.

use derive_more::{Deref, DerefMut};
use std::sync::Arc;
use waywe_ecs::{entity::EntityHashMap, prelude::*, schedule::ScheduleLabel};
use waywe_runtime::gpu::Wgpu;
use waywe_uuid::TypeUuid;

/// Links an entity in the render world to its corresponding entity in the main world.
#[derive(Component, TypeUuid, Clone, Copy)]
#[uuid = "56dfc659-ac5b-4411-87d6-ebb0d307e11b"]
pub struct MainEntity(pub Entity);

/// Schedule label for the extraction phase.
///
/// During this phase, data is transferred from the main world to the render world.
#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceneExtract;

/// Schedule label for the render phase.
///
/// During this phase, GPU operations are performed to render the scene.
#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Render;

/// System sets for organizing the render schedule.
#[derive(SystemSet, Debug, PartialEq, Eq, Default, Clone, Copy, Hash)]
pub enum RenderSet {
    /// Update render systems.
    #[default]
    Update,
    /// Prepare for rendering.
    PrepareRender,
    /// Clear the rendering surface.
    ClearPass,
    /// Perform rendering operations.
    Render,
    /// Apply post-proccess effects
    ApplyEffects,
}

/// GPU resources available to the render world.
#[derive(Resource, TypeUuid, Clone, Deref, DerefMut)]
#[uuid = "07df9612-f895-4155-9df6-52f46f778832"]
pub struct RenderGpu(pub Arc<Wgpu>);

/// Maps entities from the main world to the render world.
#[derive(Resource, TypeUuid, Default, Clone, Deref, DerefMut)]
#[uuid = "d9942e39-d234-4ddf-82f2-f441519a467e"]
pub struct EntityMap(pub EntityHashMap<Entity>);
