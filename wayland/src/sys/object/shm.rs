use super::{Dispatch, FromProxy};
use crate::sys::{HasObjectType, ObjectType, proxy::WlProxy};

#[derive(Debug, Default)]
pub struct WlShm;

impl HasObjectType for WlShm {
    const OBJECT_TYPE: ObjectType = ObjectType::Shm;
}

impl FromProxy for WlShm {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for WlShm {}
