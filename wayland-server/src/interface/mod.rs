//! Data structs representing all wayland requests, events and enums

/// Interfaces generated via `wayland-scanner`
pub use wayland_client::interface::{
    BuildMessage, ObjectParent,
    generated::{self, WlObjectType, prelude::*},
};

/// Marker trait to distinguish requests from events
pub trait Event<'s>: BuildMessage<'s> {}
