use std::fs::File;

use daemonize::Daemonize;

use crate::{error::Result, utils};

pub fn daemonize() -> Result<()> {
    // TODO: check if running as sudo
    let username = utils::get_username().unwrap_or_else(|| "nobody".into());

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

    println!("[LOG] User {} has started the daemon successfully.", username);

    Ok(())
}
