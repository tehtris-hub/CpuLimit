//! Limit the CPU usage of a process.
//!
//! # Example
//!
//! ```no_run
//! use cpulimiter::{CpuLimit, Pid};
//!
//! let handle = CpuLimit::new(Pid::from(1048), 10.0).unwrap();
//! handle.set_limit(42.0);
//! handle.stop();
//! ```

mod error;
mod limiter;
mod pid;
mod process_group;
mod process_iterator;
mod stat_iterator;

pub use limiter::CpuLimit;
pub use pid::Pid;
