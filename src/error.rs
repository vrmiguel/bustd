use std::{any::Any, str::Utf8Error};

use daemonize::DaemonizeError;

#[derive(Debug)]
pub enum Error {
    // Only possible uname error: "buf is invalid"
    UnameFailed,
    ProcessNotFound(&'static str),
    InvalidPidSupplied,
    ProcessGroupNotFound,
    InvalidSignal,
    Io {
        reason: String,
    },
    Daemonize {
        error: DaemonizeError,
    },
    Unicode {
        error: Utf8Error,
    },
    NoPermission,

    // mlockall-specific errors
    ///
    CouldNotLockMemoryError,
    TooMuchMemoryToLockError,
    InvalidFlagsError,
    // Should not happen but better safe than sorry
    UnknownMlockallError,
    UnknownKillError,
    UnknownGetpguidError,
    ThreadError {
        error: Box<dyn Any + Send + 'static>,
    },

    #[cfg(feature = "glob-ignore")]
    GlobPattern {
        error: glob::PatternError
    },

    // Errors that are likely impossible to happen
    InvalidLinuxVersionError,
    MalformedStatmError,
    MalformedPressureFileError,
    StringFromBytesError,
    ParseIntError,
    ParseFloatError,
    SysconfFailedError,
    SysInfoFailedError,
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io {
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
        Self::Daemonize { error }
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Self {
        Self::Unicode { error }
    }
}

impl From<Box<dyn Any + Send + 'static>> for Error {
    fn from(error: Box<dyn Any + Send + 'static>) -> Self {
        Self::ThreadError { error }
    }
}

#[cfg(feature = "glob-ignore")]
impl From<glob::PatternError> for Error {
    fn from(error: glob::PatternError) -> Self {
        Self::GlobPattern { error }
    }
}