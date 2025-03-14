use super::{Dispatch, FromProxy};
use crate::sys::{HasObjectType, ObjectType, proxy::WlProxy};

#[derive(Debug, Default)]
pub struct WlShmPool;

impl HasObjectType for WlShmPool {
    const OBJECT_TYPE: ObjectType = ObjectType::ShmPool;
}

impl FromProxy for WlShmPool {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for WlShmPool {}
