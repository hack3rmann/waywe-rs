use std::num::NonZeroU32;

pub mod wire;

pub struct ObjectId(pub NonZeroU32);

pub const WL_DISPLAY: ObjectId = ObjectId(NonZeroU32::new(1).unwrap());
pub const WL_REGISTRY: ObjectId = ObjectId(NonZeroU32::new(2).unwrap());
pub const WL_COMPOSITOR: ObjectId = ObjectId(NonZeroU32::new(3).unwrap());
pub const WL_SHM: ObjectId = ObjectId(NonZeroU32::new(4).unwrap());
pub const WP_VIEWPORTER: ObjectId = ObjectId(NonZeroU32::new(5).unwrap());
pub const ZWLR_LAYER_SHELL_V1: ObjectId = ObjectId(NonZeroU32::new(6).unwrap());
