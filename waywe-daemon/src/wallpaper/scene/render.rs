use crate::runtime::{gpu::Wgpu, wayland::MonitorId};
use bevy_ecs::{prelude::*, schedule::ScheduleLabel};
use derive_more::{Deref, DerefMut};
use smallvec::SmallVec;
use std::{collections::HashMap, sync::Arc};

#[derive(Component, Clone, Copy)]
pub struct MainEntity(pub Entity);

#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceneExtract;

#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Render;

#[derive(SystemSet, Debug, PartialEq, Eq, Default, Clone, Copy, Hash)]
pub enum SceneRenderStage {
    #[default]
    Update,
    PreRender,
    Render,
    Present,
}

#[derive(Resource, Clone, Deref, DerefMut)]
pub struct RenderGpu(pub Arc<Wgpu>);

#[derive(Resource, Debug)]
pub(crate) struct QueuedPlugEvents(pub SmallVec<[MonitorPlugged; 4]>);

#[derive(Event, Clone, Copy, Debug)]
pub struct MonitorPlugged {
    pub id: MonitorId,
}

#[derive(Event, Clone, Copy)]
pub struct MonitorUnplugged {
    pub id: MonitorId,
}

#[derive(Resource, Default, Clone, Deref, DerefMut)]
pub struct EntityMap(pub HashMap<Entity, Entity>);
