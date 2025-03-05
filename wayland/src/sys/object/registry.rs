use super::Dispatch;
use crate::{
    interface::{Event as _, WlRegistryGlobalEvent}, object::ObjectId, sys::wire::Message
};
use std::{collections::HashMap, ffi::CString};

#[derive(Clone, Debug, PartialEq, Default, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct WlRegistryGlobalInfo {
    pub name: ObjectId,
    pub version: u32,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct WlRegistry {
    // TODO(hack3rmann): make it faster
    pub interfaces: HashMap<CString, WlRegistryGlobalInfo>,
}

impl Dispatch for WlRegistry {
    // TODO(hack3rmann): handle all events
    fn dispatch(&mut self, message: Message<'_>) {
        let Some(event) = WlRegistryGlobalEvent::from_message(message) else {
            return;
        };

        self.interfaces.insert(
            event.interface.to_owned(),
            WlRegistryGlobalInfo {
                name: event.name,
                version: event.version,
            },
        );
    }
}
