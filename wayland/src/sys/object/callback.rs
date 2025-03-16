use super::{Dispatch, FromProxy};
use crate::{
    object::{HasObjectType, WlObjectType},
    sys::proxy::WlProxy,
};

#[derive(Debug, Default)]
pub struct WlCallback;

impl HasObjectType for WlCallback {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Callback;
}

impl FromProxy for WlCallback {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for WlCallback {}
