use libc::{c_int, mlockall};
use libc::{EAGAIN, EINVAL, ENOMEM, EPERM};
use libc::{MCL_CURRENT, MCL_FUTURE};
use no_panic::no_panic;

use crate::errno::errno;
use crate::error::{Error, Result};


// TODO: use cc crate to get MCL_ONFAULT
const MCL_ONFAULT: i32 = 4;

#[no_panic]
pub fn _mlockall_wrapper(flags: c_int) -> Result<()> {
    // Safety: mlockall is safe
    let err = unsafe { mlockall(flags) };
    if err == 0 {
        return Ok(());
    }

    // If err != 0, errno was set to describe the error that mlockall had
    Err(match errno() {
        // Some or all of the memory identified by the operation could not be locked when the call was made.
        EAGAIN => Error::CouldNotLockMemoryError,
        // The flags argument is zero, or includes unimplemented flags.
        EINVAL => Error::InvalidFlagsError,

        // Locking all of the pages currently mapped into the address space of the process
        // would exceed an implementation-defined limit on the amount of memory
        // that the process may lock.
        ENOMEM => Error::TooMuchMemoryToLockError,
        // The calling process does not have appropriate privileges to perform the requested operation
        EPERM => Error::PermissionError,
        // Should not happen
        _ => Error::UnknownMlockallError,
    })
}

pub fn lock_memory_pages() -> Result<()> {
    match _mlockall_wrapper(MCL_CURRENT | MCL_FUTURE | MCL_ONFAULT) {
        Err(err) => {
            eprintln!("First try at mlockall failed: {:?}", err);
        },
        Ok(_) => return Ok(())
    }

    _mlockall_wrapper(MCL_CURRENT | MCL_FUTURE)
}
