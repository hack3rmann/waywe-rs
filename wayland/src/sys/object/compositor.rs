use super::{Dispatch, FromProxy};
use crate::{
    object::{HasObjectType, WlObjectType},
    sys::proxy::WlProxy,
};

#[derive(Default)]
pub struct WlCompositor;

impl HasObjectType for WlCompositor {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Compositor;
}

impl FromProxy for WlCompositor {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for WlCompositor {}
