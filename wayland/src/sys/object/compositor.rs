use super::{Dispatch, FromProxy};
use crate::sys::{HasObjectType, ObjectType, proxy::WlProxy};

#[derive(Default)]
pub struct WlCompositor;

impl HasObjectType for WlCompositor {
    const OBJECT_TYPE: ObjectType = ObjectType::Compositor;
}

impl FromProxy for WlCompositor {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for WlCompositor {}
