use std::fmt;
use std::result::Result as StdResult;
use std::sync::{MutexGuard, PoisonError};
use tokio::sync::mpsc::error::SendError;

/// Error type returned by `diameter` methods.
#[derive(Debug)]
pub enum Error {
    DecodeError(String),
    EncodeError(String),
    UnknownAvpCode(u32),
    ClientError(String),
    ServerError(String),
    IoError(std::io::Error),
    TryFromSliceError(std::array::TryFromSliceError),
    LockError(String),
    SendError(String),
}

/// `Result` type used by `diameter`'s API.
pub type Result<T> = StdResult<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DecodeError(msg) => write!(f, "{}", msg),
            Error::EncodeError(msg) => write!(f, "{}", msg),
            Error::UnknownAvpCode(code) => write!(f, "Unknown AVP code: {}", code),
            Error::ClientError(msg) => write!(f, "{}", msg),
            Error::ServerError(msg) => write!(f, "{}", msg),
            Error::IoError(e) => write!(f, "{}", e),
            Error::TryFromSliceError(e) => write!(f, "{}", e),
            Error::LockError(msg) => write!(f, "{}", msg),
            Error::SendError(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Error {}

// io error
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

// try from slice error
impl From<std::array::TryFromSliceError> for Error {
    fn from(err: std::array::TryFromSliceError) -> Error {
        Error::TryFromSliceError(err)
    }
}

// lock error
impl<'a, T> From<PoisonError<MutexGuard<'a, T>>> for Error {
    fn from(err: PoisonError<MutexGuard<'a, T>>) -> Self {
        Error::LockError(format!("Lock error: {}", err))
    }
}

impl<T> From<SendError<T>> for Error {
    fn from(err: SendError<T>) -> Self {
        Error::SendError(format!("Send error: {}", err))
    }
}
