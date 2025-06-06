use std::{fmt, ops::{Deref, DerefMut}};

#[repr(transparent)]
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Hash)]
pub struct Almost<T>(Option<T>);

impl<T> Almost<T> {
    pub const fn is_init(this: &Self) -> bool {
        this.0.is_some()
    }

    pub const fn is_uninit(this: &Self) -> bool {
        this.0.is_none()
    }

    pub const fn new(value: T) -> Self {
        Self(Some(value))
    }

    pub const fn uninit() -> Self {
        Self(None)
    }

    pub fn init(this: &mut Self, value: T) -> &mut T {
        this.0.insert(value)
    }

    // TODO(hack3rmann): come up with const hack for this
    pub fn into_inner(this: Self) -> T {
        match this.0 {
            Some(value) => value,
            None => panic!("Almost<T> was uninit"),
        }
    }

    pub const fn get_ref(this: &Self) -> &T {
        match &this.0 {
            Some(value) => value,
            None => panic!("Almost<T> was uninit"),
        }
    }

    pub const fn get_mut(this: &mut Self) -> &mut T {
        match &mut this.0 {
            Some(value) => value,
            None => panic!("Almost<T> was uninit"),
        }
    }
}

impl<T> Default for Almost<T> {
    fn default() -> Self {
        Self::uninit()
    }
}

impl<T: fmt::Debug> fmt::Debug for Almost<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(value) => fmt::Debug::fmt(&value, f),
            None => f.write_str("nil"),
        }
    }
}

impl<T> Deref for Almost<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Self::get_ref(self)
    }
}

impl<T> DerefMut for Almost<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::get_mut(self)
    }
}

impl<T> AsRef<T> for Almost<T> {
    fn as_ref(&self) -> &T {
        Self::get_ref(self)
    }
}
