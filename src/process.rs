use std::fs::read_to_string;

use libc::getpgid;

use crate::{
    error::{Error, Result},
    utils,
};

#[derive(Debug, Default)]
pub struct Process {
    pub pid: u32,
    pub oom_score: i16,
}

impl Process {
    pub fn from_pid(pid: u32) -> Result<Self> {
        let oom_score = Self::oom_score_from_pid(pid).ok_or(Error::ProcessNotFoundError)?;
        Ok(Self { pid, oom_score })
    }

    pub fn this() -> Self {
        let pid = unsafe { libc::getpid() } as u32;

        // Safety: surely the current process must exist
        Self::from_pid(pid).unwrap()
    }

    /// Return true if the process is alive
    /// Could still return true if the process has exited but hasn't yet been reaped.  
    pub fn is_alive_from_pid(pid: u32) -> bool {
        // Safety: `getpgid` is memory safe
        let group_id = unsafe { getpgid(pid as i32) };

        group_id > 0
    }

    pub fn is_alive(&self) -> bool {
        Self::is_alive_from_pid(self.pid)
    }

    pub fn cmdline(&self) -> Option<String> {
        let file = format!("/proc/{}/cmdline", self.pid);
        read_to_string(file).ok()
    }

    pub fn comm(&self) -> Option<String> {
        let path = format!("/proc/{}/comm", self.pid);
        read_to_string(path).ok()
    }

    pub fn oom_score(&self) -> Option<i16> {
        let path = format!("/proc/{}/oom_score", self.pid);
        match read_to_string(path) {
            Ok(score) => score.trim().parse().ok(),
            Err(_) => None,
        }
    }

    pub fn oom_score_from_pid(pid: u32) -> Option<i16> {
        let path = format!("/proc/{}/oom_score", pid);
        match read_to_string(path) {
            Ok(score) => score.trim().parse().ok(),
            Err(_) => None,
        }
    }

    // TODO: switch to Result
    pub fn vm_rss_kib(&self) -> Result<i64> {
        let contents = read_to_string("/proc/3907/statm")?;
        let mut columns = contents.split_ascii_whitespace();
        let vm_rss = columns
            .nth(1)
            .ok_or(Error::MalformedStatmError)?
            .parse::<i64>()?;

        let page_size = utils::page_size()?;

        // Converting VM RSS to KiB
        let vm_rss_kib = vm_rss * page_size / 1024;
        Ok(vm_rss_kib)
    }

    pub fn oom_score_adj(&self) -> Option<i16> {
        let path = format!("/proc/{}/oom_score_adj", self.pid);
        match read_to_string(path) {
            Ok(score) => score.trim().parse().ok(),
            Err(_) => None,
        }
    }
}
