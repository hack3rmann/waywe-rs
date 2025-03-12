use super::Dispatch;
use crate::sys::{HasObjectType, ObjectType};

#[derive(Debug)]
pub struct WlOutput;

impl HasObjectType for WlOutput {
    const OBJECT_TYPE: ObjectType = ObjectType::Output;
}

impl Dispatch for WlOutput {}
