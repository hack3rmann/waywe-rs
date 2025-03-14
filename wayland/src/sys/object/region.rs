use super::Dispatch;
use crate::sys::{HasObjectType, ObjectType};

#[derive(Debug, Default)]
pub struct WlRegion;

impl HasObjectType for WlRegion {
    const OBJECT_TYPE: ObjectType = ObjectType::Region;
}

impl Dispatch for WlRegion {}
