use super::Dispatch;
use crate::{sys::HasObjectType, sys::ObjectType};

#[derive(Debug)]
pub struct WlBuffer;

impl HasObjectType for WlBuffer {
    const OBJECT_TYPE: ObjectType = ObjectType::Buffer;
}

impl Dispatch for WlBuffer {}
