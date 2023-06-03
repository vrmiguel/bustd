use std::ffi::OsStr;
use std::fs::File;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;
use std::{ffi::CStr, mem, ptr, str};

use libc::_SC_PAGESIZE;
use libc::{getpgid, sysconf, EINVAL, EPERM, ESRCH};
use libc::{getpwuid_r, passwd};
use memchr::memchr;

use crate::errno::errno;
use crate::error::{Error, Result};

/// This macro is used whenever we call a C function but
/// strongly believe that it cannot cause any memory unsafety.
#[macro_export]
macro_rules! checked_ffi {
    ($e: expr) => {
        unsafe { $e }
    };
}

/// Gets the effective user ID of the calling process
fn effective_user_id() -> u32 {
    // Safety: the POSIX Programmer's Manual states that
    // geteuid will always be successful.
    checked_ffi! { libc::geteuid() }
}

/// Gets the process group of the process
/// with the given PID.
pub fn get_process_group(pid: i32) -> Result<i32> {
    let pgid = checked_ffi! { getpgid(pid) };
    if pgid == -1 {
        return Err(match errno() {
            EPERM => Error::NoPermission,
            ESRCH => Error::ProcessGroupNotFound,
            EINVAL => Error::InvalidPidSupplied,
            _ => Error::UnknownGetpguid,
        });
    }

    Ok(pgid)
}

/// Checks if the program is running with sudo permissions.
pub fn running_as_sudo() -> bool {
    effective_user_id() == 0
}

/// Get the size of the system's memory page in bytes.
pub fn page_size() -> Result<i64> {
    // _SC_PAGESIZE is defined in POSIX.1
    // Safety: no memory unsafety can arise from `sysconf`
    let page_size = checked_ffi! { sysconf(_SC_PAGESIZE) };
    if page_size == -1 {
        return Err(Error::SysConfFailed);
    }

    #[allow(clippy::useless_conversion)]
    // The type of page_size differs between architectures
    // so we use .into() to convert to i64 if necessary
    Ok(page_size.into())
}

/// Attempt to get the user's username from the system's password bank
pub fn get_username() -> Option<String> {
    let mut buf = [0; 2048];
    let mut result = ptr::null_mut();
    let mut passwd: passwd = unsafe { mem::zeroed() };

    let uid = effective_user_id();

    let getpwuid_r_code =
        unsafe { getpwuid_r(uid, &mut passwd, buf.as_mut_ptr(), buf.len(), &mut result) };

    if getpwuid_r_code == 0 && !result.is_null() {
        // If getpwuid_r succeeded, let's get the username from it
        let username = unsafe { CStr::from_ptr(passwd.pw_name) };
        let username = String::from_utf8_lossy(username.to_bytes());

        return Some(username.into());
    }

    None
}

fn bytes_until_first_nil(buf: &[u8]) -> &[u8] {
    let first_nul_idx = memchr(0, buf).unwrap_or(buf.len());

    &buf[0..first_nul_idx]
}

/// Construct a string slice ranging from the first position to the position of the first nul byte
pub fn str_from_bytes(buf: &[u8]) -> Result<&str> {
    let bytes = bytes_until_first_nil(buf);

    Ok(str::from_utf8(bytes)?)
}

fn path_from_bytes(buf: &[u8]) -> &Path {
    let bytes = bytes_until_first_nil(buf);

    Path::new(OsStr::from_bytes(bytes))
}

/// Given a slice of bytes, try to interpret it as a file path and open the corresponding file.
pub fn file_from_buffer(buf: &[u8]) -> Result<File> {
    let path = path_from_bytes(buf);
    let file = File::open(path)?;
    Ok(file)
}

pub fn bytes_to_megabytes(bytes: impl Into<u64>, mem_unit: impl Into<u64>) -> u64 {
    const B_TO_MB: u64 = 1000 * 1000;
    bytes.into() / B_TO_MB * mem_unit.into()
}

#[cfg(test)]
mod tests {
    use super::str_from_bytes;

    #[test]
    fn should_construct_string_slice_from_bytes() {
        assert_eq!(str_from_bytes(b"ABC\0").unwrap(), "ABC");
        assert_eq!(str_from_bytes(b"ABC\0abc").unwrap(), "ABC");
    }
}
