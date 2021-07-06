use std::ffi::CStr;

use crate::{mem_info::MemoryInfo, process::Process, utils::clear_u8};

mod daemon;
mod error;
mod kill;
mod linux_version;
mod mem_info;
mod monitor;
mod process;
mod uname;
mod utils;

fn main() -> error::Result<()> {
    let uname_data = uname::UnameData::gather()?;
    let version = uname_data.version();
    dbg!(&version);

    println!("{}", MemoryInfo::new()?);

    let mut buf = [0_u8; 50];

    let process = Process::this();
    process.comm_wip(&mut buf)?;
    let comm = utils::str_from_u8(&buf)?;

    dbg!(comm);
    // daemon::daemonize()?;

    println!("Daemon started successfully");
    println!("{}", uname_data);

    let victim = kill::choose_victim()?;
    // kill::kill_and_wait(victim)?;
    Ok(())
}
