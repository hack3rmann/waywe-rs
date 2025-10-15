//! Rendering system components and schedules.
//!
//! This module provides the core components and schedules for the render world.
//! It handles GPU operations, entity mapping between worlds, and monitor events.

use derive_more::{Deref, DerefMut};
use std::sync::Arc;
use waywe_ecs::{entity::EntityHashMap, prelude::*, schedule::ScheduleLabel};
use waywe_runtime::gpu::Wgpu;

/// Links an entity in the render world to its corresponding entity in the main world.
#[derive(Component, Clone, Copy)]
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
#[derive(Resource, Clone, Deref, DerefMut)]
pub struct RenderGpu(pub Arc<Wgpu>);

/// Maps entities from the main world to the render world.
#[derive(Resource, Default, Clone, Deref, DerefMut)]
pub struct EntityMap(pub EntityHashMap<Entity>);
