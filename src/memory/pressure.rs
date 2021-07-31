use std::fs::File;
use std::io::Read;

use crate::error::{Error, Result};
use crate::utils::str_from_u8;

macro_rules! malformed {
    () => {
        Error::MalformedPressureFileError
    };
}

/// Returns the avg10 value in the `some` row of `/proc/pressure/memory`, which
/// indicates the absolute stall time (in us) in which at least some tasks were stalled.
///
/// The data we're reading looks like:
/// ```some avg10=0.00 avg60=0.00 avg300=0.00 total=0```
///
pub fn pressure_some_avg10(mut buf: &mut [u8]) -> Result<f32> {
    let mut file = File::open("/proc/pressure/memory")?;
    buf.fill(0);

    // `buf` won't be large enough to fit all of `/proc/pressure/memory`
    // but will be large enough to hold at least the first line, which has the datas we want
    let _ = file.read(&mut buf)?;
    let contents = str_from_u8(buf)?;
    let line = contents.lines().next().ok_or(malformed!())?;
    let mut words = line.split_ascii_whitespace();
    if let Some(indicator) = words.next() {
        // This has to be the case but checking to be sure
        if indicator == "some" {
            let entry = words.next().ok_or(malformed!())?;

            // The entry is of the form `avg10=0.00`
            // We'll break this string in two in order to parse the value on the right-hand side
            let equals_pos = entry.find('=').ok_or(malformed!())?;
            let avg10 = entry.get(equals_pos + 1..).ok_or(malformed!())?;
            let avg10: f32 = avg10.trim().parse()?;
            return Ok(avg10);
        }
    }

    Err(malformed!())?
}
