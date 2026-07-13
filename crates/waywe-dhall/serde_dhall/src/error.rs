use dhall::error::Error as DhallError;
use std::fmt;
use thiserror::Error;

/// Alias for a `Result` with the error type `serde_dhall::Error`.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when deserializing Dhall data.
#[derive(Debug, Error)]
pub enum Error {
    /// Dhall error
    #[error(transparent)]
    Dhall(DhallError),
    #[error("{0}")]
    /// Error during deserializing
    Deserialize(String),
    /// Error during serializing
    #[error("{0}")]
    Serialize(String),
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error::Deserialize(msg.to_string())
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error::Serialize(msg.to_string())
    }
}
