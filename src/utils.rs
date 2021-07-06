use libc::sysconf;
use libc::_SC_PAGESIZE;

use crate::error::{Error, Result};

pub fn page_size() -> Result<i64> {
    let page_size = unsafe { sysconf(_SC_PAGESIZE) };
    if page_size == -1 {
        return Err(Error::PageSizeFailed);
    }

    Ok(page_size)
}
