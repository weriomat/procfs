use crate::from_iter;
use crate::ProcError;
use crate::ProcResult;
use std::io::BufRead;
use std::io::Read;
use std::time::Duration;

#[cfg(feature = "serde1")]
use serde::{Deserialize, Serialize};

/// Provides scheduler statistics of the CPU, based on the `/proc/schedstat` file.
///
/// To fully understand these fields, please see the [sched-stats.txt](https://www.kernel.org/doc/Documentation/scheduler/sched-stats.txt)
/// kernel documentation.
///
/// (Requires CONFIG_SCHED_INFO)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
pub struct KernelSchedStats {
    /// The version of the data provided
    pub version: u8,
    /// Timestamp when sample was collected
    pub time: u64,
    /// The statistics about the runqueue per CPU
    pub cpu: Vec<SchedstatRunqueue>,
}

impl crate::FromBufRead for KernelSchedStats {
    fn from_buf_read<R: BufRead>(r: R) -> ProcResult<Self> {
        let mut lines = r.lines();

        let mut timestamp = None;
        let mut cpu = Vec::new();

        let version_string = expect!(lines.next())?;
        let version = from_str!(u8, expect!(version_string.strip_prefix("version ")));

        for line in lines {
            let line = line?;

            if let Some(stripped) = line.strip_prefix("timestamp ") {
                timestamp = Some(from_str!(u64, stripped));
            } else if let Some(stripped) = line.strip_prefix("cpu") {
                // Only version since 15 are compatible
                if version >= 15 {
                    cpu.push(SchedstatRunqueue::from_str(stripped)?);
                }
            } else if let Some(stripped) = line.strip_prefix("domain") {
                continue;
            }
        }

        Ok(KernelSchedStats {
            version,
            time: expect!(timestamp),
            cpu,
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize))]
/// Provides scheduler statistics of the runqueue per CPU.
///
/// The format is only compatible with version 15, 16 and 17
/// Version 15 was introduced in v2.6.30 with [this commit](https://github.com/torvalds/linux/commit/67aa0f767af488a7f1e41cccb4f7a4893f24a1ab)
pub struct SchedstatRunqueue {
    // cpu<N> 1 2 3 4 5 6 7 8 9
    /// The number of times `sched_yield()` was called
    pub sched_yield: u64,
    // 2 This field is a legacy array expiration count field used in the O(1) scheduler. We kept it for ABI compatibility, but it is always set to zero.
    /// The number of times schedule() was called
    pub schedule: u64,
    /// The number of times schedule() left the processor idle
    pub schedule_idle: u64,
    /// The number of times try_to_wake_up() was called
    pub try_to_wake_up: u64,
    /// The number of times try_to_wake_up() was called to wake up the local cpu
    pub try_to_wake_up_local: u64,
    /// The sum of all time spent running by tasks on this processor (in nanoseconds)
    pub all_tasks_runtime: u64,
    /// The sum of all time spent waiting to run by tasks on this processor (in nanoseconds)
    pub all_tasks_waittime: u64,
    /// The number of timeslices run on this cpu
    pub timeslices: u64,
}

impl SchedstatRunqueue {
    fn from_str(s: &str) -> ProcResult<Self> {
        let mut s = s.split_whitespace();

        // Skip the `<N>` part (`cpu` was already stripped)
        s.next();

        let sched_yield = from_str!(u64, expect!(s.next()));

        // unused field
        s.next();
        let schedule = from_str!(u64, expect!(s.next()));
        let schedule_idle = from_str!(u64, expect!(s.next()));
        let try_to_wake_up = from_str!(u64, expect!(s.next()));
        let try_to_wake_up_local = from_str!(u64, expect!(s.next()));
        let all_tasks_runtime = from_str!(u64, expect!(s.next()));
        let all_tasks_waittime = from_str!(u64, expect!(s.next()));
        let timeslices = from_str!(u64, expect!(s.next()));
        Ok(Self {
            sched_yield,
            schedule,
            schedule_idle,
            try_to_wake_up,
            try_to_wake_up_local,
            all_tasks_runtime,
            all_tasks_waittime,
            timeslices,
        })
    }

    pub fn all_tasks_runtime(&self) -> Duration {
        Duration::from_nanos(self.all_tasks_runtime)
    }

    pub fn all_tasks_waittime(&self) -> Duration {
        Duration::from_nanos(self.all_tasks_waittime)
    }
}

