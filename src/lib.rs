//! A tool to limit the CPU usage of a process.
//!
//! This is a rewrite of the original [`cpulimit`](https://github.com/opsengine/cpulimit).
//!
//! # Example
//!
//! ```no_run
//! use cpulimit::Pid;
//!
//! Pid::from(1048).limit(10.0, Default::default());
//! ```

mod pid;
mod process_group;
mod process_iterator;
mod stat_iterator;

pub use pid::Pid;
pub use process_group::ChildrenMode;
