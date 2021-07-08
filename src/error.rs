use std::str::Utf8Error;

use daemonize::DaemonizeError;

#[derive(Debug)]
pub enum Error {
    // Only possible uname error: "buf is invalid"
    UnameError,
    ProcessNotFoundError,
    IoError { reason: String },
    DaemonizeError { error: DaemonizeError },
    UnicodeError { error: Utf8Error },
    PermissionError,

    // mlockall-specific errors
    /// 
    CouldNotLockMemoryError,
    TooMuchMemoryToLockError,
    InvalidFlagsError,
    // Should not happen but better safe than sorry
    UnknownMlockallError,

    // Errors that are likely impossible to happen
    InvalidLinuxVersionError,
    MalformedStatmError,
    MalformedPressureFileError,
    ParseIntError,
    ParseFloatError,
    NoProcessToKillError,
    SysconfFailedError,
    SysInfoFailedError,
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

impl From<std::num::ParseFloatError> for Error {
    fn from(_: std::num::ParseFloatError) -> Self {
        Self::ParseFloatError
    }
}

impl From<DaemonizeError> for Error {
    fn from(error: DaemonizeError) -> Self {
        Self::DaemonizeError { error }
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Self {
        Self::UnicodeError { error }
    }
}
