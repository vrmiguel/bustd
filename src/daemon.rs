use std::fs::OpenOptions;

use daemonize::Daemonize;

use crate::{error::Result, utils};

pub fn daemonize() -> Result<()> {

    let running_as_sudo = utils::running_as_sudo();

    let username = if running_as_sudo {
        "root".into()
    } else {
        utils::get_username().unwrap_or_else(|| "nobody".into())
    };

    let open_opts = OpenOptions::new()
        .truncate(false)
        .create(true)
        .write(true)
        .to_owned();
    
    let (stdout_path, stderr_path, pidfile_path) = if running_as_sudo {
        ("/var/log/bustd.out", "/var/log/bustd.err", "/var/run/bustd.pid")
    } else {
        ("/tmp/bustd.out", "/tmp/bustd.err", "/tmp/bustd.pid")
    };
    
    let stdout = open_opts.open(stdout_path)?;
    let stderr = open_opts.open(stderr_path)?;

    let daemonize = Daemonize::new()
        .user(&*username)
        .pid_file(pidfile_path)
        .chown_pid_file(false)
        .working_directory("/tmp")
        .stdout(stdout)
        .stderr(stderr);

    daemonize.start()?;

    println!("[LOG] User {} has started the daemon successfully.", username);

    Ok(())
}
