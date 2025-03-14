use super::{Dispatch, FromProxy};
use crate::sys::{HasObjectType, ObjectType, proxy::WlProxy};

#[derive(Debug, Default)]
pub struct WlrLayerShellV1;

impl HasObjectType for WlrLayerShellV1 {
    const OBJECT_TYPE: ObjectType = ObjectType::WlrLayerShellV1;
}

impl FromProxy for WlrLayerShellV1 {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for WlrLayerShellV1 {}
