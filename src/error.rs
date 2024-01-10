use std::fmt;

#[derive(Debug)]
pub enum Error {
    DecodeError(String),
    EncodeError(String),
    UnknownAvpCode(u32),
    IoError(std::io::Error),
    TryFromSliceError(std::array::TryFromSliceError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DecodeError(msg) => write!(f, "{}", msg),
            Error::EncodeError(msg) => write!(f, "{}", msg),
            Error::UnknownAvpCode(code) => write!(f, "Unknown AVP code: {}", code),
            Error::IoError(e) => write!(f, "{}", e),
            Error::TryFromSliceError(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<std::array::TryFromSliceError> for Error {
    fn from(err: std::array::TryFromSliceError) -> Error {
        Error::TryFromSliceError(err)
    }
}
