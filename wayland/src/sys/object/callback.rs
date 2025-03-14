use super::Dispatch;
use crate::sys::{HasObjectType, ObjectType};

#[derive(Debug, Default)]
pub struct WlCallback;

impl HasObjectType for WlCallback {
    const OBJECT_TYPE: ObjectType = ObjectType::Callback;
}

impl Dispatch for WlCallback {}
