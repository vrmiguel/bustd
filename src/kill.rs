use std::fs;
use std::time::Duration;
use std::{ffi::OsStr, time::Instant};

use libc::kill;
use libc::{SIGKILL, SIGTERM};

use crate::error::{Error, Result};
use crate::process::Process;

pub fn choose_victim(mut proc_buf: &mut [u8], mut buf: &mut [u8]) -> Result<Process> {
    let now = Instant::now();

    let mut processes = fs::read_dir("/proc/")?
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            entry
                .path()
                .file_name()
                .unwrap_or_else(|| &OsStr::new("0"))
                .to_str()
                .unwrap_or_else(|| "0")
                .trim()
                .parse::<u32>()
                .ok()
        })
        .filter(|pid| *pid > 1)
        .filter_map(|pid| Process::from_pid(pid, &mut proc_buf).ok());


    let first_process = processes.next();
    if first_process.is_none() {
        // Likely an impossible scenario but we found no process to kill!
        return Err(Error::NoProcessToKillError);
    }

    let mut victim = first_process.unwrap();
    // TODO: find another victim if victim.vm_rss_kib() fails here
    let mut victim_vm_rss_kib = victim.vm_rss_kib(&mut buf)?;

    for process in processes {
        if victim.oom_score > process.oom_score {
            // Our current victim is less innocent than the process being analysed
            continue;
        }

        let cur_vm_rss_kib = process.vm_rss_kib(&mut buf)?;
        if cur_vm_rss_kib == 0 {
            // Current process is a kernel thread
            continue;
        }

        if process.oom_score == victim.oom_score && cur_vm_rss_kib <= victim_vm_rss_kib {
            continue;
        }

        let cur_oom_score_adj = match process.oom_score_adj(&mut buf) {
            Ok(oom_score_adj) => oom_score_adj,
            Err(_) => continue,
        };

        if cur_oom_score_adj == -1000 {
            // Follow the behaviour of the standard OOM killer: don't kill processes with oom_score_adj equals to -1000
            continue;
        }

        // eprintln!("[DBG] New victim with PID={}!", process.pid);
        victim = process;
        victim_vm_rss_kib = cur_vm_rss_kib;
    }

    println!("[LOG] Found victim in {} secs.", now.elapsed().as_secs());
    println!(
        "[LOG] Victim => pid: {}, comm: {}, oom_score: {}",
        victim.pid,
        victim
            .comm(&mut buf)
            .unwrap_or_else(|_| "unknown".into())
            .trim(),
        victim.oom_score
    );

    Ok(victim)
}

pub fn kill_process(process: &Process, signal: i32) -> Result<()> {
    let res = unsafe { kill(process.pid as i32, signal) };
    // TODO: check for res
    Ok(())
}

/// Tries to kill a process and wait for it to exit
/// Will first send the victim a SIGTERM and escalate to SIGKILL if necessary
/// Returns Ok(true) if the victim was successfully terminated
pub fn kill_and_wait(process: Process) -> Result<bool> {
    let pid = process.pid;
    let now = Instant::now();

    let _ = kill_process(&process, SIGTERM);

    let half_a_sec = Duration::from_secs_f32(0.5);
    let mut sigkill_sent = false;

    for _ in 0..20 {
        std::thread::sleep(half_a_sec);
        if !process.is_alive() {
            println!("[LOG] Process with PID {} has exited.\n", pid);
            return Ok(true);
        }
        if !sigkill_sent {
            let _ = kill_process(&process, SIGKILL);
            sigkill_sent = true;
            println!(
                "[LOG] Escalated to SIGKILL after {} nanosecs",
                now.elapsed().as_nanos()
            );
        }
    }

    Ok(false)
}
