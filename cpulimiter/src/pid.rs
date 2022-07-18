//! Handle processes described by their PID.

use std::fmt::Display;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

use lazy_static::lazy_static;

use crate::process_group::ChildrenMode;
use crate::process_group::ProcessGroup;
use crate::stat_iterator::StatFile;

/// The granularity of the control slice.
///
/// The monitoring thread will wake up every `SLICE_DURATION` to compute
/// the length of the next work slice for the monitored process(es).
pub const SLICE_DURATION: Duration = Duration::from_millis(100);

lazy_static!(
    /// The number of clock ticks per second.
    ///
    /// This is a kernel constant (fixed at compile-time).
    // SAFETY: Inherently unsafe as a syscall, but the parameter is valid.
    static ref CLOCK_TICKS: i64 = unsafe {
        libc::sysconf(libc::_SC_CLK_TCK)
    };
);

/// Linux signals
#[allow(clippy::upper_case_acronyms)]
pub enum Signal {
    /// Pause the process in its current state.
    SIGSTOP,
    /// Resume the process execution.
    SIGCONT,
}

/// A process running on the system.
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Hash, Debug)]
pub struct Pid(u32);

/// The PID of the `init` daemon process.
const INIT: Pid = Pid(1);

impl FromStr for Pid {
    type Err = core::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Pid(s.parse::<u32>()?))
    }
}

impl TryFrom<&str> for Pid {
    type Error = core::num::ParseIntError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Pid::from_str(value)
    }
}

impl From<u32> for Pid {
    fn from(pid: u32) -> Self {
        Self(pid)
    }
}

impl Display for Pid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Pid {
    /// Retrieves the parent process identifier (`ppid`).
    #[must_use]
    pub fn get_ppid(&self) -> Self {
        StatFile::open(*self)
            .ok()
            .and_then(|stat| {
                let mut stat = stat.iter();
                stat.nth(3).map(ToOwned::to_owned)
            })
            .and_then(|ppid| Self::from_str(&ppid).ok())
            .unwrap_or(Self(0))
    }

    /// Indicates whether `self` is a child of `other`.
    pub fn is_child_of(&self, other: Pid) -> bool {
        let mut ppid = *self;

        while ppid > INIT && ppid != other {
            ppid = ppid.get_ppid();
        }

        ppid == other
    }

    /// Retrieves the current CPU time, sum of the `utime` (user mode) and `stime` (kernel mode).
    pub fn get_cputime(&self) -> Duration {
        StatFile::open(*self)
            .ok()
            .map(|stat| {
                let stat = stat.iter();
                let time: u64 = stat
                    .skip(13)
                    .take(2) // utime and stime (unit: clock ticks)
                    .map(|t| t.parse::<u64>().unwrap_or_default())
                    .sum();
                Duration::from_secs_f64(time as f64 / *CLOCK_TICKS as f64)
            })
            .unwrap_or(Duration::from_secs(0))
    }

    /// Actively limits the CPU time of the target process (and its children if asked to).
    fn start_limit(self, limit: f64, children_mode: ChildrenMode) {
        let limit = limit / 100_f64;
        let mut group = ProcessGroup::new(self, children_mode);
        let mut working_rate = 1_f64;

        loop {
            let cpu_usage = group.update().cpu_usage();

            working_rate *= limit / cpu_usage;
            working_rate = f64::min(working_rate, 1_f64);

            let work_time = SLICE_DURATION.mul_f64(working_rate);
            let sleep_time = SLICE_DURATION - work_time;

            group.resume();
            thread::sleep(work_time);

            group.suspend();
            thread::sleep(sleep_time);
        }
    }

    /// Actively limits the CPU time of the target process only.
    #[inline]
    pub fn limit(&self, limit: f64) {
        self.start_limit(limit, ChildrenMode::Exclude);
    }

    /// Actively limits the CPU time of the target process and its children.
    #[inline]
    pub fn limit_with_children(&self, limit: f64) {
        self.start_limit(limit, ChildrenMode::Include);
    }

    /// Sends `signal` to the process.
    #[inline]
    pub fn kill(&self, signal: &Signal) {
        let sig = match signal {
            Signal::SIGSTOP => libc::SIGSTOP,
            Signal::SIGCONT => libc::SIGCONT,
        };

        // SAFETY: Inherently unsafe as a syscall but the PID and the signal are valid values.
        unsafe { libc::kill(self.0 as _, sig) };
    }
}
