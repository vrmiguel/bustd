use crate::error::Result;
use crate::mem_info::MemoryInfo;

pub struct Monitor {
    memory_info: MemoryInfo,
}

impl Monitor {
    /// Determines how much oomf should sleep
    /// This function is essentially a copy of what earlyoom does
    /// Credits: https://github.com/rfjakob/earlyoom/blob/dea92ae67997fcb1a0664489c13d49d09d472d40/main.c#L365
    pub fn sleep_time_ms(&self) {
        // Maximum expected memory fill rate as seen
        // with `stress -m 4 --vm-bytes 4G`
        const MEM_FILL_RATE: i32 = 6000;
        // Maximum expected swap fill rate as seen
        // with membomb on zRAM
        const SWAP_FILL_RATE: i32 = 800;

        const MIN_SLEEP: i32 = 100;
        const MAX_SLEEP: i32 = 1000;
        // let available_memory_percent =
    }

    pub fn new() -> Result<Self> {
        let memory_info = MemoryInfo::new()?;

        Ok(Self { memory_info })
    }

    pub fn poll() -> Result<()> {
        Ok(())
    }
}
