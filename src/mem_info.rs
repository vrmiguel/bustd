use libc::sysinfo;

use std::{fmt, mem};

use crate::error::{Error, Result};

#[derive(Debug, Default)]
pub struct MemoryInfo {
    pub total_ram_mb: u64,
    pub total_swap_mb: u64,
    pub available_ram_mb: u64,
    pub available_swap_mb: u64,
    pub available_ram_percent: u8,
    pub available_swap_percent: u8,
}

pub fn sys_info() -> Result<sysinfo> {
    let mut sys_info: sysinfo = unsafe { mem::zeroed() };

    let ret_val = unsafe { libc::sysinfo(&mut sys_info) };

    if ret_val != 0 {
        Err(Error::SysInfoFailedError)?;
    }

    Ok(sys_info)
}

impl MemoryInfo {
    pub fn new() -> Result<MemoryInfo> {
        let sys_info = sys_info()?;

        let mem_unit = sys_info.mem_unit;
        // Converts bytes into megabytes
        const B_TO_MB: u64 = 1000 * 1000;
        let bytes_to_megabytes = |bytes| (bytes / B_TO_MB) * (mem_unit as u64);
        let ratio = |x, y| ((x as f32 / y as f32) * 100.0) as u8;

        let available_ram_mb = bytes_to_megabytes(sys_info.freeram);
        let total_ram_mb = bytes_to_megabytes(sys_info.totalram);
        let total_swap_mb = bytes_to_megabytes(sys_info.totalswap);
        let available_swap_mb = bytes_to_megabytes(sys_info.freeswap);

        let available_memory_percent = ratio(available_ram_mb, total_ram_mb);
        let available_swap_percent = if total_swap_mb != 0 {
            ratio(available_swap_mb, total_swap_mb)
        } else {
            0
        };

        Ok(MemoryInfo {
            total_ram_mb,
            available_ram_mb,
            total_swap_mb,
            available_swap_mb,
            available_ram_percent: available_memory_percent,
            available_swap_percent,
        })
    }
}

impl fmt::Display for MemoryInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Total RAM: {} MB", self.total_ram_mb)?;
        writeln!(
            f,
            "Available RAM: {} MB ({}%)",
            self.available_ram_mb, self.available_ram_percent
        )?;
        writeln!(f, "Total swap: {} MB", self.total_swap_mb)?;
        writeln!(
            f,
            "Available swap: {} MB ({} %)",
            self.available_swap_mb, self.available_swap_percent
        )
    }
}
