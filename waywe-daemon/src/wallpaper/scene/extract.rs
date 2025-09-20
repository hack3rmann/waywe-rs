use bevy_ecs::{component::Tick, system::{ReadOnlySystemParam, Res, SystemMeta, SystemParam, SystemParamItem, SystemParamValidationError, SystemState}, world::{unsafe_world_cell::UnsafeWorldCell, World}};

use crate::wallpaper::scene::MainWorld;

pub struct Extract<'w, 's, P>
where
    P: ReadOnlySystemParam + 'static,
{
    item: SystemParamItem<'w, 's, P>,
}

pub struct ExtractState<P: SystemParam + 'static> {
    state: SystemState<P>,
    main_world_state: <Res<'static, MainWorld> as SystemParam>::State,
}

// SAFETY: The only `World` access (`Res<MainWorld>`) is read-only.
unsafe impl<P> ReadOnlySystemParam for Extract<'_, '_, P> where P: ReadOnlySystemParam {}

// SAFETY: The only `World` access is properly registered by `Res<MainWorld>::init_state`.
// This call will also ensure that there are no conflicts with prior params.
unsafe impl<P> SystemParam for Extract<'_, '_, P>
where
    P: ReadOnlySystemParam,
{
    type State = ExtractState<P>;
    type Item<'w, 's> = Extract<'w, 's, P>;

    fn init_state(world: &mut World, system_meta: &mut SystemMeta) -> Self::State {
        let mut main_world = world.resource_mut::<MainWorld>();

        ExtractState {
            state: SystemState::new(&mut main_world),
            main_world_state: Res::<MainWorld>::init_state(world, system_meta),
        }
    }

    #[inline]
    unsafe fn validate_param(
        state: &Self::State,
        _system_meta: &SystemMeta,
        world: UnsafeWorldCell,
    ) -> Result<(), SystemParamValidationError> {
        // SAFETY: Read-only access to world data registered in `init_state`.
        let result = unsafe { world.get_resource_by_id(state.main_world_state) };

        let Some(main_world) = result else {
            return Err(SystemParamValidationError::invalid::<Self>(
                "`MainWorld` resource does not exist",
            ));
        };

        // SAFETY: Type is guaranteed by `SystemState`.
        let main_world: &World = unsafe { main_world.deref() };

        // SAFETY: We provide the main world on which this system state was initialized on.
        unsafe {
            SystemState::<P>::validate_param(
                &state.state,
                main_world.as_unsafe_world_cell_readonly(),
            )
        }
    }

    #[inline]
    unsafe fn get_param<'w, 's>(
        state: &'s mut Self::State,
        system_meta: &SystemMeta,
        world: UnsafeWorldCell<'w>,
        change_tick: Tick,
    ) -> Self::Item<'w, 's> {
        // SAFETY:
        // - The caller ensures that `world` is the same one that `init_state` was called with.
        // - The caller ensures that no other `SystemParam`s will conflict with the accesses we have registered.
        let main_world = unsafe {
            Res::<MainWorld>::get_param(
                &mut state.main_world_state,
                system_meta,
                world,
                change_tick,
            )
        };

        let item = state.state.get(main_world.into_inner());

        Extract { item }
    }
}

impl<'w, 's, P> std::ops::Deref for Extract<'w, 's, P>
where
    P: ReadOnlySystemParam,
{
    type Target = SystemParamItem<'w, 's, P>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'w, 's, P> std::ops::DerefMut for Extract<'w, 's, P>
where
    P: ReadOnlySystemParam,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

impl<'a, 'w, 's, P> IntoIterator for &'a Extract<'w, 's, P>
where
    P: ReadOnlySystemParam,
    &'a SystemParamItem<'w, 's, P>: IntoIterator,
{
    type Item = <&'a SystemParamItem<'w, 's, P> as IntoIterator>::Item;
    type IntoIter = <&'a SystemParamItem<'w, 's, P> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.item).into_iter()
    }
}
