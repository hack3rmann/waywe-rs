use crate::{
    runtime::{
        gpu::Wgpu,
        wayland::{MonitorId, Wayland},
    },
    wallpaper::scene::{
        image::ImagePlugin,
        material::MaterialPlugin,
        mesh::MeshPlugin,
        time::{Time, update_time},
        transform::TransformPlugin,
        video::VideoPlugin,
    },
};
use bevy_ecs::{prelude::*, schedule::ScheduleLabel, system::ScheduleSystem};
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

pub struct Renderer {
    pub world: World,
}

impl Renderer {
    pub fn new(gpu: Arc<Wgpu>, wayland: &Wayland) -> Self {
        let mut world = World::new();

        world.init_resource::<Time>();
        world.init_resource::<EntityMap>();
        world.insert_resource(RenderGpu(gpu));
        world.add_schedule(Schedule::new(SceneExtract));

        let mut render_schedule = Schedule::new(Render);
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

        let queued_plug_events = wayland
            .client_state
            .monitors
            .read()
            .unwrap()
            .keys()
            .map(|&id| MonitorPlugged { id })
            .collect();

        world.insert_resource(QueuedPlugEvents(queued_plug_events));

        let mut this = Self { world };

        // FIXME: other way to do default plugins
        this.add_plugin(TransformPlugin);
        this.add_plugin(MeshPlugin);
        this.add_plugin(ImagePlugin);
        this.add_plugin(MaterialPlugin);
        this.add_plugin(VideoPlugin);

        this
    }

    pub fn apply_queued(&mut self) {
        let Some(QueuedPlugEvents(events)) = self.world.remove_resource::<QueuedPlugEvents>()
        else {
            return;
        };

        for event in events {
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

// TODO: impl Plugin for T: RenderPlugin
pub trait RenderPlugin {
    fn init(self, renderer: &mut Renderer);
}

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
