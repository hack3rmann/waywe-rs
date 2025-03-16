use super::{Dispatch, FromProxy};
use crate::{
    object::{HasObjectType, WlObjectType},
    sys::proxy::WlProxy,
};

#[derive(Debug, Default)]
pub struct WpViewport;

impl HasObjectType for WpViewport {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Viewport;
}

impl FromProxy for WpViewport {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for WpViewport {}
