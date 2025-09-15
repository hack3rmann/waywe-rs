use crate::{
    runtime::{
        gpu::Wgpu,
        wayland::{MonitorId, Wayland},
    },
    wallpaper::scene::{Time, render_test::RenderMesh, update_time},
};
use bevy_ecs::{prelude::*, schedule::ScheduleLabel, system::ScheduleSystem};
use derive_more::{Deref, DerefMut};
use std::sync::Arc;

#[derive(Component, Clone, Copy)]
pub struct MainEntity(pub Entity);

#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceneExtract;

#[derive(ScheduleLabel, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceneRender;

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

pub struct SceneRenderer {
    pub queued_plug_events: Vec<MonitorPlugged>,
    pub world: World,
}

impl SceneRenderer {
    pub fn new(gpu: Arc<Wgpu>, wayland: &Wayland) -> Self {
        let mut world = World::new();

        world.init_resource::<Time>();
        world.insert_resource(RenderGpu(gpu));
        world.add_schedule(Schedule::new(SceneExtract));

        let mut render_schedule = Schedule::new(SceneRender);
        render_schedule.configure_sets(
            (
                SceneRenderStage::Update,
                SceneRenderStage::PreRender,
                SceneRenderStage::Render,
                SceneRenderStage::Present,
            )
                .chain(),
        );
        render_schedule.add_systems(update_time.in_set(SceneRenderStage::Update));

        world.add_schedule(render_schedule);

        let mut queued_plug_events = Vec::new();

        for &id in wayland.client_state.monitors.read().unwrap().keys() {
            queued_plug_events.push(MonitorPlugged { id });
        }

        Self { world, queued_plug_events }
    }

    pub fn apply_queued(&mut self) {
        for event in self.queued_plug_events.drain(..) {
            self.world.trigger(event);
        }
    }

    pub fn add_plugin(&mut self, plugin: impl RenderPlugin) -> &mut Self {
        plugin.init(self);
        self
    }

    pub fn add_systems<M>(
        &mut self,
        label: impl ScheduleLabel,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        let mut schedules = self.world.get_resource_or_init::<Schedules>();
        schedules.add_systems(label, systems);
        self
    }

    pub fn trigger(&mut self, event: impl Event) {
        self.world.trigger(event);
    }
}

pub trait RenderPlugin {
    fn init(self, renderer: &mut SceneRenderer);
}

#[derive(Event, Clone, Copy, Debug)]
pub struct MonitorPlugged {
    pub id: MonitorId,
}

#[derive(Event, Clone, Copy)]
pub struct MonitorUnplugged {
    pub id: MonitorId,
}
