use daemonize::DaemonizeError;

#[derive(Debug)]
pub enum Error {
    // Only possible uname error: "buf is invalid"
    UnameError,
    ProcessNotFoundError,
    IoError { reason: String },
    DaemonizeError { error: DaemonizeError },

    // Errors that are likely impossible to happen
    InvalidLinuxVersionError,
    MalformedStatmError,
    ParseIntError,
    NoProcessToKillError,
    PageSizeFailedError,
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

impl From<DaemonizeError> for Error {
    fn from(error: DaemonizeError) -> Self {
        Self::DaemonizeError { error }
    }
}
