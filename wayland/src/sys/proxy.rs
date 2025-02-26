use super::ffi::wl_proxy;
use crate::interface::Request;
use std::ptr::NonNull;

pub struct WlProxy {
    pub raw: NonNull<wl_proxy>,
}

impl WlProxy {
    fn from_request<R: Request>() -> Self {
        todo!()
    }
}
