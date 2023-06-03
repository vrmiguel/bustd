use std::{any::Any, str::Utf8Error};

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
        error: daemonize::Error,
    },
    Unicode {
        error: Utf8Error,
    },
    NoPermission,

    // mlockall-specific errors
    CouldNotLockMemory,
    TooMuchMemoryToLock,
    InvalidFlags,
    // Should not happen but better safe than sorry
    UnknownMlockall,
    UnknownKill,
    UnknownGetpguid,
    Thread {
        error: Box<dyn Any + Send + 'static>,
    },

    #[cfg(feature = "glob-ignore")]
    GlobPattern {
        error: glob::PatternError,
    },

    // Errors that are likely impossible to happen
    InvalidLinuxVersion,
    MalformedStatm,
    MalformedPressureFile,
    StringFromBytes,
    ParseInt,
    ParseFloat,
    SysConfFailed,
    SysInfoFailed,
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
        Self::ParseInt
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(_: std::num::ParseFloatError) -> Self {
        Self::ParseFloat
    }
}

impl From<daemonize::Error> for Error {
    fn from(error: daemonize::Error) -> Self {
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
        Self::Thread { error }
    }
}

#[cfg(feature = "glob-ignore")]
impl From<glob::PatternError> for Error {
    fn from(error: glob::PatternError) -> Self {
        Self::GlobPattern { error }
    }
}
