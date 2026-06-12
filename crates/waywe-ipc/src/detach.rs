use bincode::{
    BorrowDecode, Decode, Encode,
    de::{BorrowDecoder, Decoder},
    enc::Encoder,
    error::{DecodeError, EncodeError},
};
use rustix::{
    fs::{self, Mode},
    io::Errno,
};
use std::{
    fmt::{self, Debug, Display},
    fs::File,
    mem,
    os::{fd::AsFd, unix::prelude::BorrowedFd},
    path::{Path, PathBuf},
};
use thiserror::Error;
use uuid::Uuid;

pub const BINCODE_CONFIG: bincode::config::Configuration = bincode::config::standard();

#[derive(Debug, Clone, Encode, Decode, Error)]
pub enum DaemonSetupError {
    #[error("waywe-daemon panicked:\n{message}")]
    Panicked { message: String },
    #[error("failed to acquire daemon pid file lock")]
    DaemonPidLock {
        #[source]
        errno: ErrnoBincode,
    },
}

pub type DaemonSetupResult = Result<(), DaemonSetupError>;

pub struct SetupPipe {
    pub path: PathBuf,
}

impl SetupPipe {
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        let base_dir = base_dir.as_ref();

        std::fs::create_dir_all(base_dir).unwrap();

        let mut path = PathBuf::new();

        loop {
            let uuid = Uuid::now_v7();

            path.clear();
            path.push(base_dir);
            path.push(format!("fifo-{}", uuid.hyphenated()));

            match fs::mkfifoat(AtFdCwd, &path, Mode::RUSR | Mode::WUSR) {
                Ok(()) => return Self { path },
                Err(Errno::EXIST) => continue,
                Err(error) => panic!("failed to create fifo: {error}"),
            }
        }
    }

    pub fn read(&self) -> File {
        File::options().read(true).open(&self.path).unwrap()
    }
}

impl Drop for SetupPipe {
    fn drop(&mut self) {
        _ = std::fs::remove_file(&self.path);
    }
}

struct AtFdCwd;

impl AsFd for AtFdCwd {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(libc::AT_FDCWD) }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Error)]
pub struct ErrnoBincode(pub Errno);

impl ErrnoBincode {
    pub const fn as_u16(self) -> u16 {
        unsafe { mem::transmute::<Self, u16>(self) }
    }
}

impl Encode for ErrnoBincode {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.as_u16().encode(encoder)
    }
}

impl<C> Decode<C> for ErrnoBincode {
    fn decode<D: Decoder<Context = C>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let inner = u16::decode(decoder)?;
        let errno = Errno::from_raw_os_error((inner as u32 as i32).wrapping_neg());
        Ok(Self(errno))
    }
}

impl<'de, C> BorrowDecode<'de, C> for ErrnoBincode {
    fn borrow_decode<D: BorrowDecoder<'de, Context = C>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        let inner = u16::borrow_decode(decoder)?;
        let errno = Errno::from_raw_os_error((inner as u32 as i32).wrapping_neg());
        Ok(Self(errno))
    }
}

impl Debug for ErrnoBincode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for ErrnoBincode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl From<Errno> for ErrnoBincode {
    fn from(value: Errno) -> Self {
        Self(value)
    }
}

impl From<ErrnoBincode> for Errno {
    fn from(value: ErrnoBincode) -> Self {
        value.0
    }
}
