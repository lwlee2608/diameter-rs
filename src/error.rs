use std::fmt;
use std::result::Result as StdResult;
use std::sync::{MutexGuard, PoisonError};

/// Error type returned by `diameter` methods.
#[derive(Debug)]
pub enum Error {
    DecodeError(String),
    EncodeError(String),
    UnknownAvpCode(u32),
    UnknownAvpName(String),
    ClientError(String),
    ServerError(String),
    IoError(std::io::Error),
    TryFromSliceError(std::array::TryFromSliceError),
    LockError(String),
    NativeTlsError(native_tls::Error),
}

/// `Result` type used by `diameter`'s API.
pub type Result<T> = StdResult<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DecodeError(msg) => write!(f, "{}", msg),
            Error::EncodeError(msg) => write!(f, "{}", msg),
            Error::UnknownAvpCode(code) => write!(f, "Unknown AVP code: {}", code),
            Error::UnknownAvpName(name) => write!(f, "Unknown AVP name: {}", name),
            Error::ClientError(msg) => write!(f, "{}", msg),
            Error::ServerError(msg) => write!(f, "{}", msg),
            Error::IoError(e) => write!(f, "{}", e),
            Error::TryFromSliceError(e) => write!(f, "{}", e),
            Error::LockError(msg) => write!(f, "{}", msg),
            Error::NativeTlsError(e) => write!(f, "{}", e),
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

impl From<native_tls::Error> for Error {
    fn from(err: native_tls::Error) -> Self {
        Error::NativeTlsError(err)
    }
}
