use std::ffi::CStr;
use std::mem;

use crate::checked_ffi;
use crate::error::{Error, Result};
use crate::linux_version::LinuxVersion;
use libc::{uname, utsname};

pub struct Uname {
    uts_struct: utsname,
}

impl Uname {
    pub fn new() -> Result<Self> {
        // Safety: libc::utsname is a bunch of char arrays and therefore
        //         can be safely zeroed.
        let mut uts_struct: utsname = unsafe { mem::zeroed() };

        let ret_val = checked_ffi! { uname(&mut uts_struct) };

        // uname returns a negative number upon failure
        if ret_val < 0 {
            return Err(Error::UnameFailed);
        }

        Ok(Self { uts_struct })
    }

    pub fn print_info(&self) -> Result<()> {
        // Safety: dereference of these raw pointers are safe since we know they're not NULL, since
        // the buffers in struct utsname are all correctly allocated in the stack at this moment
        let sysname = unsafe { CStr::from_ptr(self.uts_struct.sysname.as_ptr()) };
        let hostname = unsafe { CStr::from_ptr(self.uts_struct.nodename.as_ptr()) };
        let release = unsafe { CStr::from_ptr(self.uts_struct.release.as_ptr()) };
        let arch = unsafe { CStr::from_ptr(self.uts_struct.machine.as_ptr()) };

        let sysname = sysname.to_str()?;
        let hostname = hostname.to_str()?;
        let release = release.to_str()?;
        let arch = arch.to_str()?;

        println!("OS:           {}", sysname);
        println!("Hostname:     {}", hostname);
        println!("Version:      {}", release);
        println!("Architecture: {}", arch);

        Ok(())
    }

    pub fn parse_version(&self) -> Result<LinuxVersion> {
        let release = unsafe { CStr::from_ptr(self.uts_struct.release.as_ptr()) };
        let release = release.to_str()?;

        LinuxVersion::from_str(release).ok_or(Error::InvalidLinuxVersion)
    }
}
