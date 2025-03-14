use super::Dispatch;
use crate::sys::{HasObjectType, ObjectType};

#[derive(Debug, Default)]
pub struct WlrLayerShellV1;

impl HasObjectType for WlrLayerShellV1 {
    const OBJECT_TYPE: ObjectType = ObjectType::WlrLayerShellV1;
}

impl Dispatch for WlrLayerShellV1 {}
