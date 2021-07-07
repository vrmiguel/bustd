use std::time::Duration;

use crate::error::Result;
use crate::memory::MemoryInfo;

pub struct Monitor {
    memory_info: MemoryInfo,
}

impl Monitor {
    /// Determines how much oomf should sleep
    /// This function is essentially a copy of how earlyoom calculates its sleep time
    ///
    /// Credits: https://github.com/rfjakob/earlyoom/blob/dea92ae67997fcb1a0664489c13d49d09d472d40/main.c#L365
    /// MIT Licensed
    pub fn sleep_time_ms(&self) -> Duration {
        // Maximum expected memory fill rate as seen
        // with `stress -m 4 --vm-bytes 4G`
        const RAM_FILL_RATE: i64 = 6000;
        // Maximum expected swap fill rate as seen
        // with membomb on zRAM
        const SWAP_FILL_RATE: i64 = 800;

        // Maximum and minimum time to sleep, in ms.
        const MIN_SLEEP: i64 = 100;
        const MAX_SLEEP: i64 = 1000;

        // TODO: make these percentages configurable by args./config. file
        const RAM_TERMINAL_PERCENT: f64 = 10.;
        const SWAP_TERMINAL_PERCENT: f64 = 10.;

        let ram_headroom_kib = (self.memory_info.available_ram_percent as f64
            - RAM_TERMINAL_PERCENT)
            * (self.memory_info.total_ram_mb as f64 * 10.0);
        let swap_headroom_kib = (self.memory_info.available_swap_percent as f64
            - SWAP_TERMINAL_PERCENT)
            * (self.memory_info.total_swap_mb as f64 * 10.0);

        let ram_headroom_kib = i64::max(ram_headroom_kib as i64, 0);
        let swap_headroom_kib = i64::max(swap_headroom_kib as i64, 0);

        let time_to_sleep = ram_headroom_kib / RAM_FILL_RATE + swap_headroom_kib / SWAP_FILL_RATE;
        // let time_to_sleep = i64::min(time_to_sleep, MAX_SLEEP);
        let time_to_sleep = i64::max(time_to_sleep, MIN_SLEEP);

        Duration::from_millis(time_to_sleep as u64)
    }

    pub fn new() -> Result<Self> {
        let memory_info = MemoryInfo::new()?;

        Ok(Self { memory_info })
    }

    pub fn low_memory(&self) {

    }

    pub fn poll(&self) -> Result<()> {
        loop {
            let sleep_time = self.sleep_time_ms();
            eprintln!("Sleeping {}ms", sleep_time.as_millis());
            
            std::thread::sleep(sleep_time);
        }
        Ok(())
    }
}
