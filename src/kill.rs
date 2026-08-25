use std::ffi::OsStr;
use std::fs::{self, DirEntry};
use std::os::unix::ffi::OsStrExt;
use std::time::Duration;
use std::time::Instant;

use libc::kill;
use libc::{EINVAL, EPERM, ESRCH, SIGKILL, SIGTERM};

use crate::errno::errno;
use crate::error::{Error, Result};
use crate::process::Process;
use crate::{cli, utils};

pub fn choose_victim(
    proc_buf: &mut [u8],
    buf: &mut [u8],
    args: &cli::CommandLineArgs,
) -> Result<Process> {
    let now = Instant::now();

    let processes = fs::read_dir("/proc/")?
        .filter_map(|e| e.ok())
        .filter_map(parse_pid_entry)
        .filter(|pid| *pid > 1)
        .filter_map(|pid| Process::from_pid(pid, proc_buf).ok());

    let mut victim: Option<(Process, i64)> = None;

    for process in processes {
        if let Some((current, _)) = &victim {
            if current.oom_score > process.oom_score {
                // Our current victim is less innocent than the process being analysed
                continue;
            }
        }

        #[cfg(feature = "glob-ignore")]
        {
            if let Some(patterns) = &args.ignored {
                match process.is_unkillable(buf, patterns) {
                    Ok(true) => continue,
                    Ok(false) => {}
                    Err(err) => {
                        if args.verbose {
                            eprintln!(
                                "Failed to determine whether PID {} is unkillable: {err:?}",
                                process.pid
                            );
                        }
                        continue;
                    }
                }
            }
        }

        let cur_vm_rss_kib = match process.vm_rss_kib(buf) {
            Ok(vm_rss_kib) => vm_rss_kib,
            Err(err) => {
                if args.verbose {
                    eprintln!("Failed to fetch vm_rss_kib of {}: {err:?}", process.pid);
                }
                continue;
            }
        };
        if cur_vm_rss_kib == 0 {
            // Current process is a kernel thread
            continue;
        }

        let cur_oom_score_adj = match process.oom_score_adj(buf) {
            Ok(oom_score_adj) => oom_score_adj,
            Err(err) => {
                if args.verbose {
                    eprintln!("Failed to fetch oom_score_adj of {}: {err:?}", process.pid);
                }
                continue;
            }
        };

        if cur_oom_score_adj == -1000 {
            // Follow the behaviour of the standard OOM killer: don't kill processes with oom_score_adj equals to -1000
            continue;
        }

        let should_replace = match &victim {
            None => true,
            Some((current, current_vm_rss_kib)) => {
                process.oom_score > current.oom_score
                    || (process.oom_score == current.oom_score
                        && cur_vm_rss_kib > *current_vm_rss_kib)
            }
        };

        if should_replace {
            victim = Some((process, cur_vm_rss_kib));
        }
    }

    let (victim, _) = victim.ok_or(Error::ProcessNotFound("choose_victim"))?;

    println!("[LOG] Found victim in {} secs.", now.elapsed().as_secs());
    println!(
        "[LOG] Victim => pid: {}, comm: {}, oom_score: {}",
        victim.pid,
        victim.comm(buf).unwrap_or("unknown").trim(),
        victim.oom_score
    );

    Ok(victim)
}

pub fn kill_process(pid: i32, signal: i32) -> Result<()> {
    let res = unsafe { kill(pid, signal) };

    if res == -1 {
        return Err(match errno() {
            // An invalid signal was specified
            EINVAL => Error::InvalidSignal,
            // Calling process doesn't have permission to send signals to any
            // of the target processes
            EPERM => Error::NoPermission,
            // The target process or process group does not exist.
            ESRCH => Error::ProcessNotFound("kill"),
            _ => Error::UnknownKill,
        });
    }

    Ok(())
}

pub fn kill_process_group(process: Process) -> Result<()> {
    let pid = process.pid;

    let pgid = utils::get_process_group(pid as i32)?;

    // TODO: kill and wait
    let _ = kill_process(-pgid, SIGTERM);

    Ok(())
}

/// Tries to kill a process and wait for it to exit
/// Will first send the victim a SIGTERM and escalate to SIGKILL if necessary
/// Returns Ok(true) if the victim was successfully terminated
pub fn kill_and_wait(process: Process) -> Result<bool> {
    let pid = process.pid;
    let now = Instant::now();

    let _ = kill_process(pid as i32, SIGTERM);

    let half_a_sec = Duration::from_secs_f32(0.5);
    let mut sigkill_sent = false;

    for _ in 0..20 {
        std::thread::sleep(half_a_sec);
        if !process.is_alive() {
            println!("[LOG] Process with PID {} has exited.\n", pid);
            return Ok(true);
        }
        if !sigkill_sent {
            let _ = kill_process(pid as i32, SIGKILL);
            sigkill_sent = true;
            println!(
                "[LOG] Escalated to SIGKILL after {} nanosecs",
                now.elapsed().as_nanos()
            );
        }
    }

    Ok(false)
}

fn parse_pid(name: &OsStr) -> Option<u32> {
    let bytes = name.as_bytes();

    if bytes.is_empty() {
        return None;
    }

    bytes.iter().try_fold(0_u32, |pid, &byte| {
        let digit = byte.checked_sub(b'0')?;
        if digit > 9 {
            return None;
        }

        pid.checked_mul(10)?.checked_add(u32::from(digit))
    })
}

fn parse_pid_entry(entry: DirEntry) -> Option<u32> {
    parse_pid(&entry.file_name())
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    use super::parse_pid;

    #[test]
    fn parses_numeric_proc_entries() {
        assert_eq!(parse_pid(OsStr::new("0")), Some(0));
        assert_eq!(parse_pid(OsStr::new("12345")), Some(12345));
        assert_eq!(parse_pid(OsStr::new("4294967295")), Some(u32::MAX));
    }

    #[test]
    fn rejects_non_numeric_proc_entries() {
        assert_eq!(parse_pid(OsStr::new("")), None);
        assert_eq!(parse_pid(OsStr::new("self")), None);
        assert_eq!(parse_pid(OsStr::new("12a")), None);
        assert_eq!(parse_pid(OsStr::new("+12")), None);
        assert_eq!(parse_pid(OsStr::new(" 12")), None);
        assert_eq!(parse_pid(OsStr::from_bytes(b"\xff")), None);
    }

    #[test]
    fn rejects_pid_overflow() {
        assert_eq!(parse_pid(OsStr::new("4294967296")), None);
    }
}
