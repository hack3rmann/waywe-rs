use super::Dispatch;
use crate::sys::{HasObjectType, ObjectType};

#[derive(Debug, Default)]
pub struct WlrLayerSurfaceV1;

impl HasObjectType for WlrLayerSurfaceV1 {
    const OBJECT_TYPE: ObjectType = ObjectType::WlrLayerSurfaceV1;
}

impl Dispatch for WlrLayerSurfaceV1 {}
