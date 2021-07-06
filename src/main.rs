use crate::{kill::kill_and_wait, process::Process};

mod error;
mod kill;
mod linux_version;
mod mem_info;
mod process;
mod uname;
mod utils;

fn main() -> error::Result<()> {
    let sysinfo = mem_info::MemoryInfo::new();
    dbg!(sysinfo);
    let uname_data = uname::UnameData::gather()?;
    dbg!(&uname_data);
    let version = uname_data.version();
    dbg!(&version);

    let proc = Process::this();
    dbg!(proc.is_alive());
    dbg!(proc.cmdline());
    dbg!(proc.comm());
    dbg!(proc.oom_score());
    dbg!(proc.oom_score_adj());
    dbg!(proc.vm_rss_kib());

    let victim = kill::choose_victim().unwrap();
    dbg!(kill_and_wait(victim));
    Ok(())
}
