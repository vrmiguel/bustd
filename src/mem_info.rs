use libc::sysinfo;

use std::mem;

#[derive(Debug, Default)]
pub struct MemoryInfo {
    pub uptime: usize,
    pub total_ram: usize,
    pub free_ram: usize,
    pub shared_ram: usize,
}

impl MemoryInfo {
    pub fn new() -> MemoryInfo {
        let mut sysinfo_s: sysinfo = unsafe { mem::zeroed() };

        let ret_val = unsafe { libc::sysinfo(&mut sysinfo_s) };

        assert_eq!(ret_val, 0, "libc::sysinfo failed.");

        MemoryInfo {
            uptime: sysinfo_s.uptime as usize,
            total_ram: sysinfo_s.totalram as usize,
            free_ram: sysinfo_s.freeram as usize,
            shared_ram: sysinfo_s.sharedram as usize,
        }
    }
}
