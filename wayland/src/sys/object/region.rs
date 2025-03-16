use super::{Dispatch, FromProxy};
use crate::{
    object::{HasObjectType, WlObjectType},
    sys::proxy::WlProxy,
};

#[derive(Debug, Default)]
pub struct WlRegion;

impl HasObjectType for WlRegion {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Region;
}

impl FromProxy for WlRegion {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for WlRegion {}
