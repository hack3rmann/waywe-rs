use std::num::NonZeroU32;

#[derive(Clone, Debug, PartialEq, Eq, Copy, PartialOrd, Ord, Hash)]
pub struct ObjectId(pub NonZeroU32);

impl ObjectId {
    pub const fn new(value: u32) -> Self {
        Self(NonZeroU32::new(value).unwrap())
    }
}

impl Default for ObjectId {
    fn default() -> Self {
        Self::new(1)
    }
}

impl From<ObjectId> for u32 {
    fn from(value: ObjectId) -> Self {
        value.0.get()
    }
}

pub const WL_DISPLAY: ObjectId = ObjectId::new(1);
pub const WL_REGISTRY: ObjectId = ObjectId::new(2);
pub const WL_COMPOSITOR: ObjectId = ObjectId::new(3);
pub const WL_SHM: ObjectId = ObjectId::new(4);
pub const WP_VIEWPORTER: ObjectId = ObjectId::new(5);
pub const ZWLR_LAYER_SHELL_V1: ObjectId = ObjectId::new(6);

#[derive(Clone, Debug, PartialEq, Default, Eq, Copy, PartialOrd, Ord, Hash)]
pub struct ObjectIdProvider {
    pub last: ObjectId,
}

impl ObjectIdProvider {
    pub const fn new() -> Self {
        Self { last: ObjectId::new(1) }
    }

    pub const fn next_id(&mut self) -> ObjectId {
        let result = self.last;
        self.last.0 = NonZeroU32::new(self.last.0.get() + 1).unwrap();
        result
    }
}
