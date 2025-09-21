use bevy_ecs::{
    bundle::Bundle,
    event::Event,
    observer::Observer,
    resource::Resource,
    schedule::{InternedSystemSet, IntoScheduleConfigs, Schedule, ScheduleLabel, Schedules},
    system::{IntoObserverSystem, ScheduleSystem},
    world::{FromWorld, Mut, World},
};

#[derive(Default, Debug)]
pub struct EcsApp {
    pub world: World,
}

impl EcsApp {
    pub fn new() -> Self {
        Self {
            world: World::new(),
        }
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

    pub fn add_schedule(&mut self, schedule: Schedule) -> &mut Self {
        let mut schedules = self.world.get_resource_or_init::<Schedules>();
        schedules.insert(schedule);
        self
    }

    pub fn insert_resource(&mut self, resource: impl Resource) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    pub fn init_resource<R: Resource + FromWorld>(&mut self) -> &mut Self {
        self.world.init_resource::<R>();
        self
    }

    pub fn get_resource<R: Resource>(&self) -> Option<&R> {
        self.world.get_resource::<R>()
    }

    pub fn resource<R: Resource>(&self) -> &R {
        self.world.resource::<R>()
    }

    pub fn get_resource_mut<R: Resource>(&mut self) -> Option<Mut<'_, R>> {
        self.world.get_resource_mut::<R>()
    }

    pub fn resource_mut<R: Resource>(&mut self) -> Mut<'_, R> {
        self.world.resource_mut::<R>()
    }

    #[track_caller]
    pub fn configure_sets<M>(
        &mut self,
        schedule: impl ScheduleLabel,
        sets: impl IntoScheduleConfigs<InternedSystemSet, M>,
    ) -> &mut Self {
        let mut schedules = self.world.resource_mut::<Schedules>();
        schedules.configure_sets(schedule, sets);
        self
    }

    pub fn add_observer<E: Event, B: Bundle, M>(
        &mut self,
        system: impl IntoObserverSystem<E, B, M>,
    ) -> &mut Self {
        self.world.spawn(Observer::new(system));
        self
    }
}
