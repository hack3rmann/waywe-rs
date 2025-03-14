use super::Dispatch;
use crate::sys::{HasObjectType, ObjectType};

#[derive(Debug, Default)]
pub struct WlShm;

impl HasObjectType for WlShm {
    const OBJECT_TYPE: ObjectType = ObjectType::Shm;
}

impl Dispatch for WlShm {}
