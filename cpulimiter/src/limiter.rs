use std::{
    sync::{
        mpsc::{self, Receiver, SyncSender},
        Arc,
    },
    thread,
    time::Duration,
};

use parking_lot::RwLock;

use crate::{
    error::Result,
    process_group::{ChildrenMode, ProcessGroup},
    Pid,
};

/// The granularity of the control slice.
///
/// The monitoring thread will wake up every `SLICE_DURATION` to compute
/// the length of the next work slice for the monitored process(es).
pub const SLICE_DURATION: Duration = Duration::from_millis(100);

/// Messages sent to the limiting thread to change its behavior.
pub enum Command {
    Limit(f64),
    Stop,
}

/// A handle to manage the CPU limit enforced on the target process.
#[derive(Clone)]
pub struct CpuLimit {
    sender: SyncSender<Command>,
    group: Arc<RwLock<ProcessGroup>>,
}

/// The limiting function, to be run in a separate thread.
fn limiter_fn(limit: f64, group: &Arc<RwLock<ProcessGroup>>, rx: &Receiver<Command>) {
    let mut limit = limit / 100_f64;
    let mut working_rate = 1_f64;

    loop {
        if let Ok(cmd) = rx.try_recv() {
            match cmd {
                Command::Limit(new_limit) => limit = new_limit,
                Command::Stop => {
                    group.read().resume();
                    break;
                }
            }
        }

        if group.write().update().is_err() {
            // bail-out if the target process is dead.
            break;
        }

        let cpu_usage = group.read().cpu_usage();
        working_rate *= limit / cpu_usage;
        working_rate = f64::min(working_rate, 1_f64);

        group.read().resume();
        let work_time = SLICE_DURATION.mul_f64(working_rate);
        thread::sleep(work_time);

        let sleep_time = SLICE_DURATION - work_time;
        group.read().suspend();
        thread::sleep(sleep_time);
    }
}

impl CpuLimit {
    /// Limits the CPU time of the target process only.
    pub fn new(pid: Pid, limit: f64) -> Result<Self> {
        Self::start_limit(pid, limit, ChildrenMode::Exclude)
    }

    /// Limits the CPU time of the target process and its children.
    pub fn new_with_children(pid: Pid, limit: f64) -> Result<Self> {
        Self::start_limit(pid, limit, ChildrenMode::Include)
    }

    /// Limits the CPU time of the target process (and its children if asked to).
    fn start_limit(pid: Pid, limit: f64, children_mode: ChildrenMode) -> Result<Self> {
        let (tx, rx) = mpsc::sync_channel(1);
        let group = ProcessGroup::new(pid, children_mode)?;
        let group = Arc::new(RwLock::new(group));

        let group_clone = group.clone();
        thread::Builder::new().spawn(move || limiter_fn(limit, &group_clone, &rx))?;

        Ok(CpuLimit { sender: tx, group })
    }

    /// Updates the limit applied to the target process.
    pub fn set_limit(&self, limit: f64) -> Result<()> {
        self.sender.send(Command::Limit(limit))?;
        Ok(())
    }

    /// Stops the limiting thread.
    pub fn stop(&self) -> Result<()> {
        self.sender.send(Command::Stop)?;
        Ok(())
    }

    /// Retrieves the CPU usage of the target process.
    pub fn cpu_usage(&self) -> f64 {
        self.group.read().cpu_usage()
    }
}
