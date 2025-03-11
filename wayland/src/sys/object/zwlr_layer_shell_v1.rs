use super::{
    Dispatch,
    registry::{HasDynamicInterface, WlDynamicInterface, WlRegistry},
};
use crate::{
    interface::registry::request::HasInterface,
    sys::{Interface, InterfaceObjectType, ffi::wl_message},
};

#[derive(Debug, Default)]
pub struct ZwlrLayerShellV1;

impl HasDynamicInterface for ZwlrLayerShellV1 {
    fn get_interface(registry: &WlRegistry) -> Option<WlDynamicInterface<'_>> {
        let name = c"zwlr_layer_shell_v1";
        let global = registry.interfaces.get(name)?;

        Some(WlDynamicInterface {
            integer_name: global.name,
            name,
            version: global.version,
            methods: &[],
            events: &[],
        })
    }
}

// impl HasInterface for ZwlrLayerShellV1 {
//     const INTERFACE: Interface = Interface {
//         object_type: InterfaceObjectType::ZwlrLayerShellV1,
//         version: 1,
//     };
// }

impl Dispatch for ZwlrLayerShellV1 {}
