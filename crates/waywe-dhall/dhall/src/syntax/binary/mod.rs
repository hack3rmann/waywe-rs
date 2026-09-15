mod decode;
mod encode;
pub use decode::Value as CBORValue;
pub use decode::decode;
pub use encode::encode;
