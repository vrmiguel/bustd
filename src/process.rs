use std::io::Read;
use std::io::Write;

use libc::getpgid;

use crate::{
    error::{Error, Result},
    utils::{self, str_from_u8},
};

#[derive(Debug, Default)]
pub struct Process {
    pub pid: u32,
    pub oom_score: i16,
}

impl Process {
    pub fn from_pid(pid: u32, buf: &mut [u8]) -> Result<Self> {
        let oom_score =
            Self::oom_score_from_pid(pid, buf).or(Err(Error::ProcessNotFound("from_pid")))?;
        Ok(Self { pid, oom_score })
    }

    #[allow(dead_code)]
    /// Returns the current process represented as a Process struct
    /// Unused in the actual code but very often used when debugging
    pub fn this(buf: &mut [u8]) -> Result<Self> {
        let pid = unsafe { libc::getpid() } as u32;

        Self::from_pid(pid, buf)
    }

    /// Return true if the process is alive
    /// Could still return true if the process has exited but hasn't yet been reaped.
    /// TODO: would it be better to check for /proc/<PID>/ in here?
    pub fn is_alive_from_pid(pid: u32) -> bool {
        // Safety: `getpgid` is memory safe
        let group_id = unsafe { getpgid(pid as i32) };

        group_id > 0
    }

    pub fn is_alive(&self) -> bool {
        Self::is_alive_from_pid(self.pid)
    }

    pub fn comm<'a>(&self, buf: &'a mut [u8]) -> Result<&'a str> {
        write!(&mut *buf, "/proc/{}/comm\0", self.pid)?;
        {
            let mut file = utils::file_from_buffer(buf)?;
            buf.fill(0);
            let _ = file.read(buf)?;
        }

        str_from_u8(buf)
    }

    pub fn oom_score_from_pid(pid: u32, buf: &mut [u8]) -> Result<i16> {
        write!(&mut *buf, "/proc/{}/oom_score\0", pid)?;
        let contents = {
            let mut file = utils::file_from_buffer(buf)?;
            buf.fill(0);
            let _ = file.read(buf)?;

            str_from_u8(buf)?.trim()
        };

        Ok(contents.parse::<i16>()?)
    }

    /// Reads VmRSS from /proc/<PID>/statm
    /// In order to match the VmRSS value in /proc/<PID>/status, we'll
    /// multiply the number of pages in `statm` by the page size of our system and then convert
    /// that value to KiB
    pub fn vm_rss_kib(&self, buf: &mut [u8]) -> Result<i64> {
        write!(&mut *buf, "/proc/{}/statm\0", self.pid)?;
        let mut columns = {
            let mut file = utils::file_from_buffer(buf)?;
            buf.fill(0);
            let _ = file.read(buf)?;

            str_from_u8(buf)?.split_ascii_whitespace()
        };
        let vm_rss = columns
            .nth(1)
            .ok_or(Error::MalformedStatm)?
            .parse::<i64>()?;

        let page_size = utils::page_size()?;

        // Converting VM RSS to KiB
        let vm_rss_kib = vm_rss * page_size / 1024;
        Ok(vm_rss_kib)
    }

    #[cfg(feature = "glob-ignore")]
    /// Checks if the process' name matches any of the given glob patterns
    pub fn is_unkillable(&self, buf: &mut [u8], patterns: &[String]) -> Result<bool> {
        use glob::Pattern;

        let comm = self.comm(buf)?.trim();
        for pattern in patterns {
            let pattern = Pattern::new(pattern)?;
            if pattern.matches(comm) {
                println!(
                    "Skipping \"{}\" since it matches an unkillable pattern",
                    comm
                );
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn oom_score_adj(&self, buf: &mut [u8]) -> Result<i16> {
        write!(&mut *buf, "/proc/{}/oom_score_adj\0", self.pid)?;
        let contents = {
            let mut file = utils::file_from_buffer(buf)?;
            buf.fill(0);
            let _ = file.read(buf)?;

            str_from_u8(buf)?.trim()
        };

        Ok(contents.parse::<i16>()?)
    }
}

#[cfg(test)]
mod tests {
    // We'll use the Process struct from procfs
    // in order to test our own Process struct.
    //
    // The reason we don't use `procfs` directly is
    // because our implementation is considerably leaner.
    use procfs;

    // Returns the Process representing the
    // process of the caller test
    fn this() -> ([u8; 100], crate::process::Process) {
        let mut buf = [0_u8; 100];
        (buf, crate::process::Process::this(&mut buf).unwrap())
    }

    #[test]
    fn comm() {
        let (mut buf, this) = this();
        let comm = this.comm(&mut buf).unwrap();

        // We'll now represent the current process using
        // the external `procfs` crate as well
        let _this = procfs::process::Process::myself().unwrap();
        let _comm = _this.stat.comm;

        assert_eq!(comm.trim(), _comm)
    }

    #[test]
    fn oom_score() {
        let (_, this) = this();

        let _this = procfs::process::Process::myself().unwrap();

        assert_eq!(this.oom_score, _this.oom_score().unwrap() as i16);
    }

    #[test]
    fn pid() {
        let (_, this) = this();

        let _this = procfs::process::Process::myself().unwrap();

        assert_eq!(this.pid as i32, _this.pid);
    }
}
