use super::{Dispatch, FromProxy};
use crate::{
    object::{HasObjectType, WlObjectType},
    sys::proxy::WlProxy,
};

#[derive(Debug, Default)]
pub struct WpViewporter;

impl HasObjectType for WpViewporter {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Viewporter;
}

impl FromProxy for WpViewporter {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for WpViewporter {}
