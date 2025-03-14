use super::{Dispatch, FromProxy};
use crate::sys::{proxy::WlProxy, HasObjectType, ObjectType};

#[derive(Debug, Default)]
pub struct WlCallback;

impl HasObjectType for WlCallback {
    const OBJECT_TYPE: ObjectType = ObjectType::Callback;
}

impl FromProxy for WlCallback {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for WlCallback {}
