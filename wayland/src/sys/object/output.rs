use super::{Dispatch, FromProxy};
use crate::{
    object::{HasObjectType, WlObjectType},
    sys::proxy::WlProxy,
};

#[derive(Debug, Default)]
pub struct WlOutput;

impl HasObjectType for WlOutput {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Output;
}

impl FromProxy for WlOutput {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for WlOutput {}
