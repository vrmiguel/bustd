use std::fs::File;

use daemonize::Daemonize;

use crate::{error::Result, utils};

pub fn daemonize() -> Result<()> {
    let username = unsafe { utils::get_username() }.unwrap_or_else(|| "nobody".into());

    // Save in ~/ instead of /tmp/ ?
    let stdout = File::create("/tmp/bustd.out")?;
    let stderr = File::create("/tmp/bustd.err")?;

    let daemonize = Daemonize::new()
        .user(&*username)
        .pid_file("/tmp/bustd.pid")
        .chown_pid_file(false)
        .working_directory("/tmp")
        .stdout(stdout)
        .stderr(stderr);

    daemonize.start()?;

    println!("[LOG] User {} has started the daemon", username);

    Ok(())
}
