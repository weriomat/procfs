use std::time::Duration;

use procfs::{prelude::*, KernelSchedStats, SchedstatRunqueue};

/// A basic example of /proc/schedstat usage.
fn main() {
    if let Ok(schedstat) = KernelSchedStats::current() {
        println!("Version: {}", schedstat.version);
        println!("Timestamp: {}", schedstat.time);

        for (id, cpu) in schedstat.cpu.iter().enumerate() {
            pretty_print(cpu, id, 25);
        }
    }
}

fn pretty_print(cpu: &SchedstatRunqueue, id: usize, width: usize) {
    println!("CPU {id}");
    println!("{:>width$}: {:>22} counts", "sched_yield", fmt_count(cpu.sched_yield));
    println!("{:>width$}: {:>22} counts", "sched_yield", fmt_count(cpu.schedule));
    println!(
        "{:>width$}: {:>22} counts",
        "schedule_idle",
        fmt_count(cpu.schedule_idle)
    );
    println!(
        "{:>width$}: {:>22} counts",
        "try_to_wake_up",
        fmt_count(cpu.try_to_wake_up)
    );
    println!(
        "{:>width$}: {:>22} counts",
        "try_to_wake_up_local",
        fmt_count(cpu.try_to_wake_up_local)
    );
    println!(
        "{:>width$}: {:>22} counts ({:>14})",
        "all_tasks_runtime",
        fmt_count(cpu.all_tasks_runtime),
        fmt_duration(cpu.all_tasks_runtime())
    );
    println!(
        "{:>width$}: {:>22} counts ({:>14})",
        "all_tasks_waittime",
        fmt_count(cpu.all_tasks_waittime),
        fmt_duration(cpu.all_tasks_waittime())
    );
    println!("{:>width$}: {:>22} counts", "timeslices", fmt_count(cpu.timeslices));
}

fn fmt_count(n: u64) -> String {
    n.to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect::<Vec<_>>()
        .join(",")
}

/// Format a duration in nanoseconds as human-readable time.
fn fmt_duration(d: Duration) -> String {
    let secs = d.as_secs();
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let rem_secs = secs % 60;
    if hours > 0 {
        format!("{}h {}m {}s", hours, mins, rem_secs)
    } else if mins > 0 {
        format!("{}m {}s", mins, rem_secs)
    } else if d.as_secs_f64() >= 1.0 {
        format!("{:.2} s", d.as_secs_f64())
    } else if d.subsec_micros() > 0 {
        format!("{:.2} ms", d.as_secs_f64() * 1000.0)
    } else {
        format!("{} ns", d.as_nanos())
    }
}
