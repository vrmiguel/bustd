use std::fs::File;
use std::{ffi::CStr, mem, ptr, str};

use libc::sysconf;
use libc::_SC_PAGESIZE;
use libc::{getpwuid_r, passwd};
use no_panic::no_panic;

use crate::error::{Error, Result};

#[no_panic]
/// Gets the effective user ID of the calling process
fn effective_user_id() -> u32 {
    // Safety: the POSIX Programmer's Manual states that
    // geteuid will always be successful.
    unsafe { libc::geteuid() }
}

#[no_panic]
pub fn running_as_sudo() -> bool {
    effective_user_id() == 0
} 

#[no_panic]
pub fn page_size() -> Result<i64> {
    let page_size = unsafe { sysconf(_SC_PAGESIZE) };
    if page_size == -1 {
        return Err(Error::SysconfFailedError);
    }

    // The type of page_size differs between architectures
    // so we use .into() to convert to i64 if necessary
    Ok(page_size.into())
}

// #[no_panic]
pub fn get_username() -> Option<String> {
    let mut buf = [0; 2048];
    let mut result = ptr::null_mut();
    let mut passwd: passwd = unsafe { mem::zeroed() };

    let uid = effective_user_id();

    let getpwuid_r_code = unsafe { 
        getpwuid_r(uid, &mut passwd, buf.as_mut_ptr(), buf.len(), &mut result)
    };

    if getpwuid_r_code == 0 && !result.is_null() {
        // If getpwuid_r succeeded, let's get the username from it
        let username = unsafe { CStr::from_ptr(passwd.pw_name) };
        let username = String::from_utf8_lossy(username.to_bytes());

        return Some(username.into());
    }

    None
}

#[no_panic]
pub fn str_from_u8(buf: &[u8]) -> Result<&str> {
    let first_nul_idx = buf.iter().position(|&c| c == b'\0').unwrap_or(buf.len());

    let bytes = buf.get(0..first_nul_idx).ok_or(Error::StringFromBytesError)?;
    
    Ok(str::from_utf8(bytes)?)
}

// pub fn file_from_buffer(buf: [u8; 50]) -> Result<([u8; 50], File)> {
//     let path = str_from_u8(&buf)?;
//     let mut file = File::open(&path)?;
//     Ok((buf, file))
// }

pub fn file_from_buffer(buf: &[u8]) -> Result<File> {
    let path = str_from_u8(&buf)?;
    let file = File::open(&path)?;
    Ok(file)
}
