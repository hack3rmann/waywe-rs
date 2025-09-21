//! Rendering system components and schedules.
//!
//! This module provides the core components and schedules for the render world.
//! It handles GPU operations, entity mapping between worlds, and monitor events.

use crate::runtime::{gpu::Wgpu, wayland::MonitorId};
use bevy_ecs::{entity::EntityHashMap, prelude::*, schedule::ScheduleLabel};
use derive_more::{Deref, DerefMut};
use smallvec::SmallVec;
use std::sync::Arc;

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
pub enum SceneRenderStage {
    /// Update render systems.
    #[default]
    Update,
    /// Prepare for rendering.
    PreRender,
    /// Perform rendering operations.
    Render,
    /// Present the rendered frame.
    Present,
}

/// GPU resources available to the render world.
#[derive(Resource, Clone, Deref, DerefMut)]
pub struct RenderGpu(pub Arc<Wgpu>);

/// Events queued for processing during initialization.
#[derive(Resource, Debug)]
pub(crate) struct QueuedPlugEvents(pub SmallVec<[MonitorPlugged; 4]>);

/// Event triggered when a monitor is plugged in.
#[derive(Event, Clone, Copy, Debug)]
pub struct MonitorPlugged {
    /// The ID of the monitor that was plugged in.
    pub id: MonitorId,
}

/// Event triggered when a monitor is unplugged.
#[derive(Event, Clone, Copy)]
pub struct MonitorUnplugged {
    /// The ID of the monitor that was unplugged.
    pub id: MonitorId,
}

/// Maps entities from the main world to the render world.
#[derive(Resource, Default, Clone, Deref, DerefMut)]
pub struct EntityMap(pub EntityHashMap<Entity>);
