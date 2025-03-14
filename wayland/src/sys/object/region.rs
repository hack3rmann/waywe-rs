use super::{Dispatch, FromProxy};
use crate::sys::{HasObjectType, ObjectType, proxy::WlProxy};

#[derive(Debug, Default)]
pub struct WlRegion;

impl HasObjectType for WlRegion {
    const OBJECT_TYPE: ObjectType = ObjectType::Region;
}

impl FromProxy for WlRegion {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for WlRegion {}
