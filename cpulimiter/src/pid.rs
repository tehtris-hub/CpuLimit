//! Handle processes described by their PID.

use std::fmt::Display;
use std::str::FromStr;
use std::time::Duration;

use lazy_static::lazy_static;

use crate::stat_iterator::StatFile;

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
    /// Check process existence.
    SIGNULL,
}

/// The representation of a process running on the system.
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

    /// Indicates whether the process is alive or not.
    pub fn alive(&self) -> bool {
        self.kill(&Signal::SIGNULL).is_ok()
    }

    /// Sends `signal` to the process.
    #[inline]
    pub(crate) fn kill(self, signal: &Signal) -> Result<(), ()> {
        let sig = match signal {
            Signal::SIGNULL => 0,
            Signal::SIGSTOP => libc::SIGSTOP,
            Signal::SIGCONT => libc::SIGCONT,
        };

        // SAFETY: Inherently unsafe as a syscall but the PID and the signal are valid values.
        let res = unsafe { libc::kill(self.0 as _, sig) };

        if res == 0 {
            Ok(())
        } else {
            Err(())
        }
    }
}
