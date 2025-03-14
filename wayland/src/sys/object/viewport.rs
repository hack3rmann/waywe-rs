use super::{Dispatch, FromProxy};
use crate::sys::{HasObjectType, ObjectType, proxy::WlProxy};

#[derive(Debug, Default)]
pub struct WpViewport;

impl HasObjectType for WpViewport {
    const OBJECT_TYPE: ObjectType = ObjectType::Viewport;
}

impl FromProxy for WpViewport {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for WpViewport {}
