use std::{ffi::OsStr, time::Instant};

use walkdir::WalkDir;

use crate::error::{Error, Result};
use crate::process::Process;

pub fn choose_victim() -> Result<Process> {
    let now = Instant::now();
    let mut processes = WalkDir::new("/proc/")
        .max_depth(1)
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
        .filter_map(|pid| Process::from_pid(pid).ok());

    let first_pid = processes.next();
    if first_pid.is_none() {
        // Likely an impossible scenario but we found no process to kill!
        return Err(Error::NoProcessToKill);
    }

    let mut victim = first_pid.unwrap();
    let mut victim_vm_rss_kib = victim.vm_rss_kib()?;

    let mut proc_counter = 1;

    for process in processes {
        if victim.oom_score > process.oom_score {
            // Our current victim is less innocent than the process being analysed
            continue;
        }

        let cur_vm_rss_kib = process.vm_rss_kib()?;
        if cur_vm_rss_kib == 0 {
            // Current process is a kernel thread
            continue;
        }

        if process.oom_score == victim.oom_score && cur_vm_rss_kib <= victim_vm_rss_kib {
            procs_jumped += 1;
            continue;
        }

        let cur_oom_score_adj = match process.oom_score_adj() {
            Some(oom_score_adj) => oom_score_adj,
            None => continue,
        };

        if cur_oom_score_adj == -1000 {
            // Follow the behaviour of the standard OOM killer: don't kill processes with oom_score_adj equals to -1000
            continue;
        }

        victim = process;
        victim_vm_rss_kib = cur_vm_rss_kib;

        proc_counter += 1;
    }

    println!("[LOG] Found victim in {} secs.", now.elapsed().as_secs());
    println!(
        "[LOG] Victim => pid: {}, comm: {}, oom_score: {}",
        victim.pid,
        victim
            .comm()
            .unwrap_or_else(|| "unknown comm".into())
            .trim(),
        victim.oom_score
    );

    println!("{} processes analyzed", proc_counter);

    Ok(victim)
}
