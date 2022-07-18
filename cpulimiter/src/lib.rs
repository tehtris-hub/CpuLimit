//! A tool to limit the CPU usage of a process.
//!
//! This is a rewrite of the original [`cpulimit`](https://github.com/opsengine/cpulimit).
//!
//! # Example
//!
//! ```no_run
//! use cpulimiter::Pid;
//!
//! // Only limit the target process
//! Pid::from(1048).limit(10.0);
//! // or also account for the children
//! Pid::from(2096).limit_with_children(42.0);
//! ```

mod pid;
mod process_group;
mod process_iterator;
mod stat_iterator;

pub use pid::Pid;
