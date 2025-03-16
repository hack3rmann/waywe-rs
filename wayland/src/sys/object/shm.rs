use super::{Dispatch, FromProxy};
use crate::{
    object::{HasObjectType, WlObjectType},
    sys::proxy::WlProxy,
};

#[derive(Debug, Default)]
pub struct WlShm;

impl HasObjectType for WlShm {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Shm;
}

impl FromProxy for WlShm {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for WlShm {}
