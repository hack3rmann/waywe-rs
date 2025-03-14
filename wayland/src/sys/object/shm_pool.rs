use super::Dispatch;
use crate::sys::{HasObjectType, ObjectType};

#[derive(Debug, Default)]
pub struct WlShmPool;

impl HasObjectType for WlShmPool {
    const OBJECT_TYPE: ObjectType = ObjectType::ShmPool;
}

impl Dispatch for WlShmPool {}
