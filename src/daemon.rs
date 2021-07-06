use std::fs::File;

use daemonize::Daemonize;

use crate::{error::Result, utils};

pub fn daemonize() -> Result<()> {
    let username = unsafe { utils::get_username() }.unwrap_or_else(|| "nobody".into());

    // Save in ~/ instead of /tmp/ ?
    let stdout = File::create("/tmp/oomfd.out")?;
    let stderr = File::create("/tmp/oomfd.err")?;

    let daemonize = Daemonize::new()
        .user(&*username)
        .pid_file("/tmp/oomfd.pid")
        .chown_pid_file(false)
        .working_directory("/tmp")
        .stdout(stdout)
        .stderr(stderr);

    daemonize.start()?;

    println!("[LOG] User {} starting the daemon.", username);

    Ok(())
}
