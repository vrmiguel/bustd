use std::ffi::CStr;
// use no_panic::no_panic;
use std::mem;

use libc::{uname, utsname, c_int};
use crate::error::{Error, Result};
use crate::linux_version::LinuxVersion;
use crate::utils::str_from_u8;

use no_panic::no_panic;

extern { fn _char_is_signed() -> c_int; }

#[no_panic]
fn char_is_signed() -> bool {
    1 == unsafe { _char_is_signed() }
}

pub struct Uname {
    uts_struct: utsname
}

impl Uname {
    pub fn new() -> Result<Self> {
        let mut uts_struct: utsname = unsafe { mem::zeroed() };

        let ret_val = unsafe { uname(&mut uts_struct) };
        // uname returns a negative number upon failure
        if ret_val < 0 {
            return Err(Error::UnameError);
        }

        Ok(
            Self {
                uts_struct
            }
        )
    }

    pub fn print_info(&self) -> Result<()> {
        // Safety: dereference of these raw pointers are safe since we know they're not NULL, since
        // the buffers in struct utsname are all correctly allocated in the stack at this moment
        
        let sysname = unsafe { CStr::from_ptr(self.uts_struct.sysname.as_ptr()) };
        let hostname = unsafe { CStr::from_ptr(self.uts_struct.nodename.as_ptr()) };
        let release = unsafe { CStr::from_ptr(self.uts_struct.release.as_ptr()) };
        let arch = unsafe { CStr::from_ptr(self.uts_struct.machine.as_ptr()) };

        let sysname = str_from_u8(sysname.to_bytes())?;
        let hostname = str_from_u8(hostname.to_bytes())?;
        let release = str_from_u8(release.to_bytes())?;
        let arch = str_from_u8(arch.to_bytes())?;

        println!("OS:           {}", sysname);
        println!("Hostname:     {}", hostname);
        println!("Version:      {}", release);
        println!("Architecture: {}", arch);

        Ok(())
    }

    pub fn parse_version(&self) -> Result<LinuxVersion> {
        let release = unsafe { CStr::from_ptr(self.uts_struct.release.as_ptr()) };
        let release = str_from_u8(release.to_bytes())?;
        
        // The position of the first dot in the 'release' string
        let dot_idx = match release.find('.') {
            Some(idx) => idx,
            None => return Err(Error::InvalidLinuxVersionError),
        };

        let (major, minor) = release.split_at(dot_idx);

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

// impl Display for UnameData {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         writeln!(f, "OS: {}", self.system_name)?;
//         writeln!(f, "Hostname: {}", self.node_name)?;
//         writeln!(f, "Version: {}", self.version)?;
//         writeln!(f, "Architecture: {}", self.machine)
//     }
// }