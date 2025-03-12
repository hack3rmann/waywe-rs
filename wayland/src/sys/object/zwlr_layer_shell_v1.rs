use super::Dispatch;
use crate::sys::{HasObjectType, ObjectType};

#[derive(Debug, Default)]
pub struct ZwlrLayerShellV1;

impl HasObjectType for ZwlrLayerShellV1 {
    const OBJECT_TYPE: ObjectType = ObjectType::WlrLayerShellV1;
}

impl Dispatch for ZwlrLayerShellV1 {}
