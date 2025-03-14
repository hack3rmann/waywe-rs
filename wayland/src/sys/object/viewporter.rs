use super::{Dispatch, FromProxy};
use crate::sys::{HasObjectType, ObjectType, proxy::WlProxy};

#[derive(Debug, Default)]
pub struct WpViewporter;

impl HasObjectType for WpViewporter {
    const OBJECT_TYPE: ObjectType = ObjectType::Viewporter;
}

impl FromProxy for WpViewporter {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for WpViewporter {}
