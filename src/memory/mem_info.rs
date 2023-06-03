use std::{fmt, mem};

use libc::sysinfo;

use crate::{
    checked_ffi,
    error::{Error, Result},
    utils::bytes_to_megabytes,
};

#[derive(Debug, Default)]
pub struct MemoryInfo {
    pub total_ram_mb: u64,
    pub total_swap_mb: u64,
    pub available_ram_mb: u64,
    pub available_swap_mb: u64,
    pub available_ram_percent: u8,
    pub available_swap_percent: u8,
}

/// Simple wrapper over libc's sysinfo
fn sys_info() -> Result<sysinfo> {
    // Safety: the all-zero byte pattern is a valid sysinfo struct
    let mut sys_info: sysinfo = unsafe { mem::zeroed() };

    // Safety: sysinfo() is safe and must not fail when passed a valid reference
    let ret_val = checked_ffi! { libc::sysinfo(&mut sys_info) };

    if ret_val != 0 {
        // The only error that sysinfo() can have happens when
        // it is supplied an invalid struct sysinfo pointer
        //
        // This error should really not happen during this function
        return Err(Error::SysInfoFailed);
    }

    Ok(sys_info)
}

impl MemoryInfo {
    pub fn new() -> Result<MemoryInfo> {
        let sysinfo {
            mem_unit,
            freeram,
            totalram,
            totalswap,
            freeswap,
            ..
        } = sys_info()?;

        let ratio = |x, y| ((x as f32 / y as f32) * 100.0) as u8;

        let available_ram_mb = bytes_to_megabytes(freeram, mem_unit);
        let total_ram_mb = bytes_to_megabytes(totalram, mem_unit);
        let total_swap_mb = bytes_to_megabytes(totalswap, mem_unit);
        let available_swap_mb = bytes_to_megabytes(freeswap, mem_unit);

        let available_ram_percent = ratio(available_ram_mb, total_ram_mb);
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
            available_ram_percent,
            available_swap_percent,
        })
    }
}

impl fmt::Display for MemoryInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
