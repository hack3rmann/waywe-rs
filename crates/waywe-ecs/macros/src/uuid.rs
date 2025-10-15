use proc_macro2::TokenStream;
use quote::quote;
use std::time::SystemTime;
use uuid::{Builder as UuidBuilder, Uuid};

pub fn generate_uuid() -> Uuid {
    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("unix epoch was a long time ago");

    let millis = ts.as_millis() as u64;
    let mut random_bytes = [0; 10];
    rand::fill(&mut random_bytes);

    UuidBuilder::from_unix_timestamp_millis(millis, &random_bytes).into_uuid()
}

pub fn quote_uuid(uuid: &Uuid) -> TokenStream {
    let bytes = uuid.as_bytes();

    quote! { [ #( #bytes ),* ] }
}
