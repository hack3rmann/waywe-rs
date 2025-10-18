use crate::{
    component::Component,
    entity::{ComponentCloneCtx, SourceComponent},
};
use core::marker::PhantomData;

/// Function type that can be used to clone a component of an entity.
pub type ComponentCloneFn = fn(&SourceComponent, &mut ComponentCloneCtx);

/// The clone behavior to use when cloning or moving a [`Component`].
#[derive(Clone, Debug, Default)]
pub enum ComponentCloneBehavior {
    /// Uses the default behavior (which is passed to [`ComponentCloneBehavior::resolve`])
    #[default]
    Default,
    /// Do not clone/move this component.
    Ignore,
    /// Uses a custom [`ComponentCloneFn`].
    Custom(ComponentCloneFn),
}

impl ComponentCloneBehavior {
    /// Set clone handler based on `Clone` trait.
    ///
    /// If set as a handler for a component that is not the same as the one used to create this handler, it will panic.
    pub fn clone<C: Component + Clone>() -> Self {
        Self::Custom(component_clone_via_clone::<C>)
    }

    /// Returns the "global default"
    pub fn global_default_fn() -> ComponentCloneFn {
        component_clone_ignore
    }

    /// Resolves the [`ComponentCloneBehavior`] to a [`ComponentCloneFn`]. If [`ComponentCloneBehavior::Default`] is
    /// specified, the given `default` function will be used.
    pub fn resolve(&self, default: ComponentCloneFn) -> ComponentCloneFn {
        match self {
            ComponentCloneBehavior::Default => default,
            ComponentCloneBehavior::Ignore => component_clone_ignore,
            ComponentCloneBehavior::Custom(custom) => *custom,
        }
    }
}

/// Component [clone handler function](ComponentCloneFn) implemented using the [`Clone`] trait.
/// Can be [set](Component::clone_behavior) as clone handler for the specific component it is implemented for.
/// It will panic if set as handler for any other component.
///
pub fn component_clone_via_clone<C: Clone + Component>(
    source: &SourceComponent,
    ctx: &mut ComponentCloneCtx,
) {
    if let Some(component) = source.read::<C>() {
        ctx.write_target_component(component.clone());
    }
}

/// Noop implementation of component clone handler function.
///
/// See [`EntityClonerBuilder`](crate::entity::EntityClonerBuilder) for details.
pub fn component_clone_ignore(_source: &SourceComponent, _ctx: &mut ComponentCloneCtx) {}

/// Wrapper for components clone specialization using autoderef.
#[doc(hidden)]
pub struct DefaultCloneBehaviorSpecialization<T>(PhantomData<T>);

impl<T> Default for DefaultCloneBehaviorSpecialization<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// Base trait for components clone specialization using autoderef.
#[doc(hidden)]
pub trait DefaultCloneBehaviorBase {
    fn default_clone_behavior(&self) -> ComponentCloneBehavior;
}

impl<C> DefaultCloneBehaviorBase for DefaultCloneBehaviorSpecialization<C> {
    fn default_clone_behavior(&self) -> ComponentCloneBehavior {
        ComponentCloneBehavior::Default
    }
}

/// Specialized trait for components clone specialization using autoderef.
#[doc(hidden)]
pub trait DefaultCloneBehaviorViaClone {
    fn default_clone_behavior(&self) -> ComponentCloneBehavior;
}

impl<C: Clone + Component> DefaultCloneBehaviorViaClone for &DefaultCloneBehaviorSpecialization<C> {
    fn default_clone_behavior(&self) -> ComponentCloneBehavior {
        ComponentCloneBehavior::clone::<C>()
    }
}
