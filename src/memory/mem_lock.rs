use libc::{c_int, mlockall};
use libc::{EAGAIN, EINVAL, ENOMEM, EPERM};
use libc::{MCL_CURRENT, MCL_FUTURE};

use crate::checked_ffi;
use crate::errno::errno;
use crate::error::{Error, Result};

extern "C" {
    pub static _MCL_ONFAULT: libc::c_int;
}

pub fn _mlockall_wrapper(flags: c_int) -> Result<()> {
    let err = checked_ffi! { mlockall(flags) };
    if err == 0 {
        return Ok(());
    }

    // If err != 0, errno was set to describe the error that mlockall had
    Err(match errno() {
        // Some or all of the memory identified by the operation could not be locked when the call was made.
        EAGAIN => Error::CouldNotLockMemory,
        // The flags argument is zero, or includes unimplemented flags.
        EINVAL => Error::InvalidFlags,

        // Locking all of the pages currently mapped into the address space of the process
        // would exceed an implementation-defined limit on the amount of memory
        // that the process may lock.
        ENOMEM => Error::TooMuchMemoryToLock,
        // The calling process does not have appropriate privileges to perform the requested operation
        EPERM => Error::NoPermission,
        // Should not happen
        _ => Error::UnknownMlockall,
    })
}

pub fn lock_memory_pages() -> Result<()> {
    // TODO: check for _MCL_ONFAULT == -1

    #[allow(non_snake_case)]
    let MCL_ONFAULT: c_int = checked_ffi! { _MCL_ONFAULT };
    match _mlockall_wrapper(MCL_CURRENT | MCL_FUTURE | MCL_ONFAULT) {
        Err(err) => {
            eprintln!("First try at mlockall failed: {:?}", err);
        }
        Ok(_) => return Ok(()),
    }

    _mlockall_wrapper(MCL_CURRENT | MCL_FUTURE)
}
