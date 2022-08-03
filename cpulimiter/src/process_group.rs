//! Track the CPU usage of a process (and its children).

use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::error::{Error, Result};
use crate::pid::{Pid, Signal};
use crate::process_iterator::ProcessIterator;

/// Whether the child processes should be monitored.
pub enum ChildrenMode {
    Include,
    Exclude,
}

impl Default for ChildrenMode {
    fn default() -> Self {
        ChildrenMode::Exclude
    }
}

/// An abstraction to compute the CPU usage of a process and its children.
pub struct ProcessGroup {
    target: Pid,
    children_mode: ChildrenMode,
    children: HashSet<Pid>,
    last_update: Instant,
    total_time: Duration,
    cpu_usage: f64,
}

impl ProcessGroup {
    /// Instantiates a process group.
    pub fn new(pid: Pid, children_mode: ChildrenMode) -> Result<Self> {
        let mut group = Self {
            target: pid,
            children: HashSet::new(),
            children_mode,
            cpu_usage: 0_f64,
            last_update: Instant::now(),
            total_time: Duration::from_secs(0),
        };

        group.update()?;
        Ok(group)
    }

    /// Computes the CPU usage since the last call and smoothly updates the value.
    pub fn update(&mut self) -> Result<()> {
        if !self.target.alive() {
            return Err(Error::DeadTarget);
        }

        let prev_time = self.total_time;
        self.total_time = self.target.get_cputime();

        if let ChildrenMode::Include = self.children_mode {
            if let Ok(processes) = ProcessIterator::new() {
                self.children.clear();
                for process in processes {
                    if process != self.target && process.is_child_of(self.target) {
                        self.children.insert(process);
                        self.total_time += process.get_cputime();
                    }
                }
            }
        }

        let consumed = self.total_time - prev_time;

        if !prev_time.is_zero() {
            let elapsed = self.last_update.elapsed();
            self.last_update = Instant::now();

            let cpu_usage = consumed.as_secs_f64() / elapsed.as_secs_f64();

            // smooth out strong fluctuations
            self.cpu_usage = 0.8 * self.cpu_usage + 0.2 * cpu_usage;
        }

        Ok(())
    }

    /// Retrieves the previously computed CPU usage.
    #[inline]
    pub fn cpu_usage(&self) -> f64 {
        self.cpu_usage
    }

    /// Sends a signal to the target process and its children if needed.
    fn kill(&self, signal: &Signal) {
        let _ = self.target.kill(signal);
        if let ChildrenMode::Include = self.children_mode {
            for child in &self.children {
                let _ = child.kill(signal);
            }
        }
    }

    /// Suspends the execution of the group.
    #[inline]
    pub fn suspend(&self) {
        self.kill(&Signal::SIGSTOP);
    }

    /// Resumes the execution of the group.
    #[inline]
    pub fn resume(&self) {
        self.kill(&Signal::SIGCONT);
    }
}
