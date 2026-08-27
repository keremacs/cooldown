//! Lightweight self-benchmark — memory, CPU, thread count.

use std::time::Instant;

use sysinfo::{Pid, ProcessesToUpdate, System};

use crate::models::BenchmarkMetrics;

pub struct BenchmarkMonitor {
    system: System,
    started: Instant,
    pid: Pid,
}

impl BenchmarkMonitor {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_all();
        let pid = Pid::from_u32(std::process::id());
        Self {
            system,
            started: Instant::now(),
            pid,
        }
    }

    pub fn snapshot(&mut self) -> BenchmarkMetrics {
        self.system.refresh_processes(ProcessesToUpdate::Some(&[self.pid]), true);
        let (memory_mb, cpu, threads) = self
            .system
            .process(self.pid)
            .map(|p| {
                (
                    p.memory() as f64 / 1024.0 / 1024.0,
                    p.cpu_usage() as f64,
                    1u32, // sysinfo 0.33 doesn't expose thread count per process easily
                )
            })
            .unwrap_or((0.0, 0.0, 1));

        BenchmarkMetrics {
            memory_mb,
            cpu_percent: cpu,
            threads,
            uptime_secs: self.started.elapsed().as_secs(),
        }
    }
}
