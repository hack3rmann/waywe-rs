use super::Dispatch;
use crate::sys::{HasObjectType, ObjectType};

#[derive(Default)]
pub struct WlCompositor;

impl HasObjectType for WlCompositor {
    const OBJECT_TYPE: ObjectType = ObjectType::Compositor;
}

impl Dispatch for WlCompositor {}
