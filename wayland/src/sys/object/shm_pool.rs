use super::{Dispatch, FromProxy};
use crate::{
    object::{HasObjectType, WlObjectType},
    sys::proxy::WlProxy,
};

#[derive(Debug, Default)]
pub struct WlShmPool;

impl HasObjectType for WlShmPool {
    const OBJECT_TYPE: WlObjectType = WlObjectType::ShmPool;
}

impl FromProxy for WlShmPool {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for WlShmPool {}
