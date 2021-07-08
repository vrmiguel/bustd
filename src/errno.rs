use cfg_if::cfg_if;
use libc::{self, c_int};

cfg_if! {
    if #[cfg(target_os = "android")] {
        unsafe fn _errno() -> *mut c_int {
            libc::__errno()
        }
    } else if #[cfg(target_os = "linux")] {
        unsafe fn _errno() -> *mut c_int {
            libc::__errno_location()
        }
    }
}

pub fn errno() -> i32 {
    unsafe { (*_errno()) as i32 }
}
