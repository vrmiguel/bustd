use crate::{memory::{MemoryInfo, lock_memory_pages}, monitor::Monitor};

mod daemon;
mod error;
mod kill;
mod linux_version;
mod memory;
mod monitor;
mod process;
mod errno;
mod uname;
mod utils;

fn main() -> error::Result<()> {
//     let uname_data = uname::UnameData::gather()?;
//     let version = uname_data.version();
//     dbg!(&version);

//     println!("{}", MemoryInfo::new()?);

    // In order to correctly use `mlockall`, we'll try our best to avoid heap allocations and
    // reuse these buffers right here, even though it makes the code less readable.
    // Buffer specific to process creation
    let proc_buf = [0_u8; 50];
    // Buffer for anything else
    let buf = [0_u8; 100];

    // daemon::daemonize()?;

    if let Err(err) = lock_memory_pages() {
        eprintln!("Failed to lock memory pages: {:?}. Continuing anyway.", err);
    } else {
        eprintln!("Memory pages locked!");
    }

    println!("Daemon started successfully");
    
    Monitor::new(proc_buf, buf)?.poll()
    // Ok(())
}
