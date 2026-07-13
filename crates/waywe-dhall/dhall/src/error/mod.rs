use std::{fmt, io};

use crate::semantics::resolve::{CyclesStack, ImportLocation};
use crate::syntax::{Import, ParseError};

mod builder;
pub use builder::*;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Parse(#[from] Box<ParseError>),
    #[error(transparent)]
    Decode(#[from] DecodeError),
    #[error(transparent)]
    Encode(#[from] EncodeError),
    #[error(transparent)]
    Resolve(#[from] ImportError),
    #[error(transparent)]
    Typecheck(#[from] TypeError),
    #[error(transparent)]
    Cache(#[from] CacheError),
}

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("missing import")]
    Missing,
    #[error("${0} env variable is not set")]
    MissingEnvVar(String),
    #[error("$HOME env variable is not set")]
    MissingHome,
    #[error("insane import")]
    SanityCheck,
    #[error("unexpected import: {0:#?}")]
    UnexpectedImport(Import<()>),
    #[error("cyclic imports, location={location:#?}")]
    ImportCycle {
        stack: CyclesStack,
        location: ImportLocation,
    },
    #[error("failed to parse url")]
    Url(#[from] url::ParseError),
}

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error(transparent)]
    CBORError(#[from] minicbor::decode::Error),
    #[error("invalid format: {0}")]
    WrongFormatError(String),
}

#[derive(Debug, Error)]
pub enum EncodeError {
    #[error(transparent)]
    CBORError(#[from] minicbor::encode::Error<core::convert::Infallible>),
}

/// A structured type error
#[derive(Debug, Error)]
#[error("type error: {message}")]
pub struct TypeError {
    message: TypeMessage,
}

/// The specific type error
#[derive(Debug)]
pub enum TypeMessage {
    Custom(String),
}

impl fmt::Display for TypeMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeMessage::Custom(custom) => f.write_str(custom),
        }
    }
}

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("missing cache config")]
    MissingConfiguration,
    #[error("failed to initialize cache")]
    Init(#[from] io::Error),
    #[error("invalid cache hash")]
    CacheHashInvalid,
}

impl TypeError {
    pub const fn new(message: TypeMessage) -> Self {
        TypeError { message }
    }
}
