use bytemuck::{Pod, Zeroable};
use smallvec::SmallVec;
use static_assertions::const_assert_eq;
use uuid::Uuid;

#[repr(C)]
#[derive(Pod, Zeroable, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventUuid(pub [u8; EventUuid::LENGTH]);

impl EventUuid {
    pub const LENGTH: usize = 16;

    pub const fn from_bytes(bytes: [u8; Self::LENGTH]) -> Self {
        Self(bytes)
    }

    pub fn generate() -> Self {
        Self(Uuid::now_v7().into_bytes())
    }

    pub const fn into_bytes(self) -> [u8; Self::LENGTH] {
        self.0
    }

    pub const fn as_bytes(&self) -> &[u8; Self::LENGTH] {
        &self.0
    }

    pub const fn as_bytes_mut(&mut self) -> &mut [u8; Self::LENGTH] {
        &mut self.0
    }
}

#[repr(transparent)]
#[derive(Pod, Zeroable, Debug, Clone, Copy)]
pub struct EventSubject(pub u16);

impl EventSubject {
    pub const DAEMON: Self = Self(0);
    pub const CLI: Self = Self(1);
    pub const EXTERNAL: Self = Self(2);
    pub const NONE: Self = Self(u16::MAX);
}

#[repr(C)]
#[derive(Pod, Zeroable, Debug, Clone, Copy)]
pub struct EventHeader {
    pub n_data_bytes: u32,
    pub source: EventSubject,
    pub destination: EventSubject,
    pub event_type: EventUuid,
    pub event_id: EventUuid,
}

#[derive(Debug)]
pub struct ExternalEvent {
    pub header: EventHeader,
    pub data: SmallVec<[u8; Self::DATA_CAPACITY]>,
}
const_assert_eq!(std::mem::size_of::<ExternalEvent>(), 128);

impl ExternalEvent {
    // NOTE(hack3rmann): 80 + smallvec-data + sizeof(EventHeader) = 128 btyes
    pub const DATA_CAPACITY: usize = 80;
}
