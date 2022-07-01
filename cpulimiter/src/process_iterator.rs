//! Parse the `/proc` directory to extract PIDs.

use std::fs;
use std::fs::ReadDir;

use crate::pid::Pid;

/// An iterator over existing processes.
pub(crate) struct ProcessIterator {
    proc: ReadDir,
}

impl ProcessIterator {
    /// Instantiates a `ProcessIterator` (open the `/proc` directory).
    pub fn new() -> Self {
        let proc = fs::read_dir("/proc").expect("Error while opening `/proc`");
        Self { proc }
    }
}

impl Iterator for ProcessIterator {
    type Item = Pid;

    /// Walks `/proc` and yields the next PID.
    ///
    /// Parsing errors are silently ignored.
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.proc.next()?.ok()?;

            let filetype = next.file_type();
            if filetype.is_err() || !filetype.unwrap().is_dir() {
                continue;
            }

            if let Some(pid) = next.file_name().to_str() {
                if let Ok(pid) = pid.parse::<u32>() {
                    return Some(Pid::from(pid));
                }
            }
        }
    }
}
