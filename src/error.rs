use std::fmt;

#[derive(Debug)]
pub enum CLientError {}

#[derive(Debug)]
pub enum Error {
    DecodeError(String),
    EncodeError(String),
    UnknownAvpCode(u32),
    ClientError(String),
    ServerError(String),
    IoError(std::io::Error),
    TryFromSliceError(std::array::TryFromSliceError),
}

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
