use box_into_inner::IntoInner as _;
use bytemuck::{Pod, Zeroable};
use smallvec::SmallVec;
use static_assertions::const_assert_eq;
use std::any::{Any, TypeId};
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

pub trait TryReplicate: Any + Send {
    fn try_replicate(&self) -> Option<Box<dyn TryReplicate>> {
        None
    }
}

impl dyn TryReplicate {
    pub fn downcast<T: Any>(self: Box<Self>) -> Result<Box<T>, Box<Self>> {
        if TypeId::of::<T>() == self.as_ref().type_id() {
            Ok((self as Box<dyn Any>).downcast::<T>().unwrap())
        } else {
            Err(self)
        }
    }
}

impl<T: Clone + Any + Send> TryReplicate for T {
    fn try_replicate(&self) -> Option<Box<dyn TryReplicate>> {
        Some(Box::new(self.clone()))
    }
}

pub struct InternalEvent(Option<Box<dyn TryReplicate>>);

impl InternalEvent {
    pub fn underlying_type(&self) -> Option<TypeId> {
        self.0.as_ref().map(Box::as_ref).map(|e| e.type_id())
    }

    /// # Safety
    ///
    /// - event should contain a value
    /// - underlying type should be exactly `E`
    pub unsafe fn downcast_unchecked<E: IntoInternalEvent>(self) -> E {
        let any = unsafe { self.0.unwrap_unchecked() };
        let boxed = unsafe { any.downcast::<E>().unwrap_unchecked() };
        boxed.into_inner()
    }

    pub async fn handle<T: IntoInternalEvent>(&mut self, f: impl AsyncFnOnce(T)) {
        // Try to replicate the event. Take the event if could not replicate
        let replicated = self.0.as_deref().and_then(TryReplicate::try_replicate);

        let Some(any_value) = replicated.or_else(|| self.0.take()) else {
            return;
        };

        let boxed_value = match any_value.downcast::<T>() {
            Ok(value) => value,
            Err(other) => {
                self.0 = Some(other);
                return;
            }
        };

        let value = boxed_value.into_inner();
        f(value).await;
    }
}

pub trait IntoInternalEvent: TryReplicate {
    fn into_event(self) -> InternalEvent
    where
        Self: Sized;
}

impl<T: TryReplicate> IntoInternalEvent for T {
    fn into_event(self) -> InternalEvent {
        InternalEvent(Some(Box::new(self)))
    }
}

pub enum Event {
    Internal(InternalEvent),
    External(ExternalEvent),
}
