#[derive(Debug)]
pub enum Error {
    // Only possible uname error: "buf is invalid"
    UnameError,
    ProcessNotFound,
    IoError { reason: String },

    // Errors that are likely impossible to happen
    InvalidLinuxVersion,
    MalformedStatm,
    ParseIntError,
    NoProcessToKill,
    // One of the folders in /proc/ has invalid Unicode
    InvalidUnicodeInProcFolder,
    PageSizeFailed,
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IoError {
            reason: err.to_string(),
        }
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(_: std::num::ParseIntError) -> Self {
        Self::ParseIntError
    }
}
