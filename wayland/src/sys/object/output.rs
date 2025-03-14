use super::{Dispatch, FromProxy};
use crate::sys::{proxy::WlProxy, HasObjectType, ObjectType};

#[derive(Debug, Default)]
pub struct WlOutput;

impl HasObjectType for WlOutput {
    const OBJECT_TYPE: ObjectType = ObjectType::Output;
}

impl FromProxy for WlOutput {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for WlOutput {}
