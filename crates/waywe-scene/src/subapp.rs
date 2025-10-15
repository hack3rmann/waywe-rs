//! ECS application wrapper for simplified world management.
//!
//! This module provides the [`EcsApp`] struct, which wraps a Bevy [`World`]
//! with convenience methods for adding systems, resources, and schedules.

use waywe_ecs::{
    bundle::Bundle,
    event::Event,
    observer::Observer,
    resource::Resource,
    schedule::{InternedSystemSet, IntoScheduleConfigs, Schedule, ScheduleLabel, Schedules},
    system::{IntoObserverSystem, ScheduleSystem},
    world::{FromWorld, Mut, World},
};

/// A wrapper around a Bevy [`World`] with convenience methods.
///
/// This struct simplifies common ECS operations like adding systems,
/// resources, and schedules.
#[derive(Default, Debug)]
pub struct EcsApp {
    /// The underlying Bevy world.
    pub world: World,
}

impl EcsApp {
    /// Create a new empty ECS app.
    pub fn new() -> Self {
        Self {
            world: World::new(),
        }
    }

    /// Add systems to a schedule.
    pub fn add_systems<M>(
        &mut self,
        label: impl ScheduleLabel,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        let mut schedules = self.world.get_resource_or_init::<Schedules>();
        schedules.add_systems(label, systems);
        self
    }

    /// Add a schedule to the app.
    pub fn add_schedule(&mut self, schedule: Schedule) -> &mut Self {
        let mut schedules = self.world.get_resource_or_init::<Schedules>();
        schedules.insert(schedule);
        self
    }

    pub fn get_resource_or_init<R: Resource + FromWorld>(&mut self) -> Mut<'_, R> {
        self.world.get_resource_or_init::<R>()
    }

    /// Insert a resource into the world.
    pub fn insert_resource(&mut self, resource: impl Resource) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    /// Initialize a resource in the world.
    pub fn init_resource<R: Resource + FromWorld>(&mut self) -> &mut Self {
        self.world.init_resource::<R>();
        self
    }

    /// Get a reference to a resource, if it exists.
    pub fn get_resource<R: Resource>(&self) -> Option<&R> {
        self.world.get_resource::<R>()
    }

    /// Get a reference to a resource, panicking if it doesn't exist.
    pub fn resource<R: Resource>(&self) -> &R {
        self.world.resource::<R>()
    }

    /// Get a mutable reference to a resource, if it exists.
    pub fn get_resource_mut<R: Resource>(&mut self) -> Option<Mut<'_, R>> {
        self.world.get_resource_mut::<R>()
    }

    /// Get a mutable reference to a resource, panicking if it doesn't exist.
    pub fn resource_mut<R: Resource>(&mut self) -> Mut<'_, R> {
        self.world.resource_mut::<R>()
    }

    /// Configure system sets for a schedule.
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

    /// Add an observer for events.
    pub fn add_observer<E: Event, B: Bundle, M>(
        &mut self,
        system: impl IntoObserverSystem<E, B, M>,
    ) -> &mut Self {
        self.world.spawn(Observer::new(system));
        self
    }
}
