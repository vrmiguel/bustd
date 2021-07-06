use crate::mem_info::MemoryInfo;

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

    daemon::daemonize()?;

    println!("Daemon started successfully");
    println!("{}", uname_data);

    let victim = kill::choose_victim()?;
    // kill::kill_and_wait(victim)?;
    Ok(())
}
