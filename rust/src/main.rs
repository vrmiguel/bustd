// use uname::Uname;

use std::ops::Not;

use linux_version::LinuxVersion;
use uname::Uname;

use crate::{error::Error, memory::lock_memory_pages, monitor::Monitor};

mod cli;
mod daemon;
mod errno;
mod error;
mod kill;
mod linux_version;
mod memory;
mod monitor;
mod process;
mod uname;
mod utils;

/// The first Linux version in which PSI information became available
const LINUX_4_20: LinuxVersion = LinuxVersion {
    major: 4,
    minor: 20,
};

fn main() -> error::Result<()> {
    let args: cli::CommandLineArgs = argh::from_env();
    let should_daemonize = args.no_daemon.not();

    // Show uname info and return the Linux version running
    {
        let ensure_msg = "Ensure you're running at least Linux 4.20";
        let uname = Uname::new()?;
        uname.print_info()?;

        match uname.parse_version() {
            Ok(version) => {
                if version < LINUX_4_20 {
                    eprintln!(
                        "{version} does not meet minimum requirements for bustd!\n{ensure_msg}"
                    );
                    return Err(Error::InvalidLinuxVersion);
                }
            }
            Err(_) => {
                eprintln!("Failed to parse Linux version!\n{ensure_msg}");
            }
        }

        if let Ok(version) = uname.parse_version() {
            if version < LINUX_4_20 {
                eprintln!("{version} does not meet minimum requirements for bustd!\n{ensure_msg}");
                return Err(Error::InvalidLinuxVersion);
            }
        } else {
            eprintln!("Failed to parse Linux version!\n{ensure_msg}");
        }
    };

    // In order to correctly use `mlockall`, we'll try our best to avoid heap allocations and
    // reuse these buffers right here, even though it makes the code less readable.
    // Buffer specific to process creation
    let proc_buf = [0_u8; 50];

    // Buffer for anything else
    let buf = [0_u8; 100];

    if should_daemonize {
        // Daemonize current process
        println!("\nStarting daemonization process!");
        daemon::daemonize()?;
    }

    // Attempt to lock the memory pages mapped to the daemon
    // in order to avoid being sent to swap when the system
    // memory is stressed
    if let Err(err) = lock_memory_pages() {
        eprintln!("Failed to lock memory pages: {:?}. Continuing anyway.", err);
    } else {
        // Save this on both bustd.out and bustd.err
        println!("Memory pages locked!");
        eprintln!("Memory pages locked!");
    }

    Monitor::new(proc_buf, buf, args)?.poll()
}
