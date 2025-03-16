use super::{Dispatch, FromProxy};
use crate::{
    object::{HasObjectType, WlObjectType},
    sys::proxy::WlProxy,
};

#[derive(Debug, Default)]
pub struct WlrLayerShellV1;

impl HasObjectType for WlrLayerShellV1 {
    const OBJECT_TYPE: WlObjectType = WlObjectType::WlrLayerShellV1;
}

impl FromProxy for WlrLayerShellV1 {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for WlrLayerShellV1 {}
