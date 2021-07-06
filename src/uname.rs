use libc::{uname, utsname};
use std::{ffi::CStr, mem};

use crate::error::{Error, Result};
use crate::linux_version::LinuxVersion;

#[derive(Debug)]
pub struct UnameData {
    pub system_name: String,
    pub node_name: String,
    pub release: String,
    pub version: String,
    pub machine: String,
}

impl UnameData {
    /// Gather's data from `uname`
    pub fn gather() -> Result<UnameData> {
        let mut uts_struct: utsname = unsafe { mem::zeroed() };

        let ret_val = unsafe { uname(&mut uts_struct) };
        // uname returns a negative number upon failure
        if ret_val < 0 {
            return Err(Error::UnameError);
        }

        let sysname_cstr = unsafe { CStr::from_ptr(uts_struct.sysname.as_ptr()) };
        let nodename_cstr = unsafe { CStr::from_ptr(uts_struct.nodename.as_ptr()) };
        let release_cstr = unsafe { CStr::from_ptr(uts_struct.release.as_ptr()) };
        let version_cstr = unsafe { CStr::from_ptr(uts_struct.version.as_ptr()) };
        let machine_cstr = unsafe { CStr::from_ptr(uts_struct.machine.as_ptr()) };

        let uname_data = UnameData {
            system_name: sysname_cstr.to_string_lossy().into_owned(),
            node_name: nodename_cstr.to_string_lossy().into_owned(),
            release: release_cstr.to_string_lossy().into_owned(),
            version: version_cstr.to_string_lossy().into_owned(),
            machine: machine_cstr.to_string_lossy().into_owned(),
        };

        Ok(uname_data)
    }

    pub fn version(&self) -> Result<LinuxVersion> {
        // The position of the first dot in the 'release' string
        let dot_idx = match self.release.find('.') {
            Some(idx) => idx,
            None => return Err(Error::InvalidLinuxVersionError),
        };

        let (major, minor) = self.release.split_at(dot_idx);

        let major = match major.parse::<u8>() {
            Ok(major) => major,
            Err(_) => return Err(Error::InvalidLinuxVersionError),
        };

        // Eat the leading dot in front of minor
        let minor = &minor[1..];
        let dot_idx = match minor.find('.') {
            Some(idx) => idx,
            None => return Err(Error::InvalidLinuxVersionError),
        };

        let minor = match (&minor[0..dot_idx]).parse::<u8>() {
            Ok(minor) => minor,
            Err(_) => return Err(Error::InvalidLinuxVersionError),
        };

        Ok(LinuxVersion { major, minor })
    }
}
