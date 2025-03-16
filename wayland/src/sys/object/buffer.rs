use super::{Dispatch, FromProxy};
use crate::{
    object::{HasObjectType, WlObjectType},
    sys::proxy::WlProxy,
};

#[derive(Debug, Default)]
pub struct WlBuffer;

impl HasObjectType for WlBuffer {
    const OBJECT_TYPE: WlObjectType = WlObjectType::Buffer;
}

impl FromProxy for WlBuffer {
    fn from_proxy(_: &WlProxy) -> Self {
        Self
    }
}

impl Dispatch for WlBuffer {}
