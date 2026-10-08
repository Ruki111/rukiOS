mod cpu;
mod disk;
mod gpu;
mod memory;
mod network;
mod processes;
mod sensors;

use std::collections::{HashMap, VecDeque};
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct Metrics {
    pub cpu_percent: Option<f64>,
    pub cores: Vec<(String, Option<f64>)>,
    pub cpu_history: Vec<u64>,
    pub memory: Option<memory::MemoryMetrics>,
    pub root_disk: Option<disk::RootDisk>,
    pub disks: Vec<disk::DiskIo>,
    pub network: Vec<network::NetworkMetric>,
    pub gpus: Vec<gpu::GpuMetrics>,
    pub temperatures: Vec<sensors::Temperature>,
    pub processes: Vec<processes::ProcessInfo>,
    pub load_average: String,
    pub uptime: String,
}

pub struct Sampler {
    previous_cpu: HashMap<String, (u64, u64)>,
    previous_network: HashMap<String, (u64, u64)>,
    previous_disks: HashMap<String, (u64, u64)>,
    previous_sample: Instant,
    cpu_history: VecDeque<u64>,
}

impl Sampler {
    pub fn new() -> Self {
        Self {
            previous_cpu: cpu::read_counters(),
            previous_network: network::read_counters()
                .into_iter()
                .map(|item| (item.name, (item.rx_bytes, item.tx_bytes)))
                .collect(),
            previous_disks: disk::read_io_counters(),
            previous_sample: Instant::now(),
            cpu_history: VecDeque::with_capacity(60),
        }
    }

    pub fn sample(&mut self) -> Metrics {
        let now = Instant::now();
        let elapsed = now
            .duration_since(self.previous_sample)
            .as_secs_f64()
            .max(0.1);
        let cpu_now = cpu::read_counters();
        let mut cores = Vec::new();
        let mut total_cpu = None;
        for (name, current) in &cpu_now {
            let usage = self
                .previous_cpu
                .get(name)
                .and_then(|previous| cpu::usage(previous, current));
            if name == "cpu" {
                total_cpu = usage;
            } else {
                cores.push((name.clone(), usage));
            }
        }
        self.previous_cpu = cpu_now;
        cores.sort_by(|a, b| a.0.cmp(&b.0));
        if let Some(usage) = total_cpu {
            self.cpu_history.push_back(usage.clamp(0.0, 100.0) as u64);
            if self.cpu_history.len() > 60 {
                self.cpu_history.pop_front();
            }
        }

        let network = network::read_counters()
            .into_iter()
            .map(|mut item| {
                if let Some((rx, tx)) = self.previous_network.get(&item.name) {
                    item.rx_bytes_per_sec =
                        Some((item.rx_bytes.saturating_sub(*rx) as f64 / elapsed) as u64);
                    item.tx_bytes_per_sec =
                        Some((item.tx_bytes.saturating_sub(*tx) as f64 / elapsed) as u64);
                }
                item
            })
            .collect::<Vec<_>>();
        self.previous_network = network
            .iter()
            .map(|item| (item.name.clone(), (item.rx_bytes, item.tx_bytes)))
            .collect();

        let disk_io = disk::read_io_counters();
        let disks = disk_io
            .iter()
            .map(|(name, (read, written))| {
                let rates = self
                    .previous_disks
                    .get(name)
                    .map(|(prev_read, prev_written)| {
                        (
                            (read.saturating_sub(*prev_read) as f64 / elapsed) as u64,
                            (written.saturating_sub(*prev_written) as f64 / elapsed) as u64,
                        )
                    });
                disk::DiskIo {
                    name: name.clone(),
                    read_bytes_per_sec: rates.map(|rate| rate.0),
                    write_bytes_per_sec: rates.map(|rate| rate.1),
                }
            })
            .collect();
        self.previous_disks = disk_io;
        self.previous_sample = now;

        Metrics {
            cpu_percent: total_cpu,
            cores,
            cpu_history: self.cpu_history.iter().copied().collect(),
            memory: memory::read(),
            root_disk: disk::read_root(),
            disks,
            network,
            gpus: gpu::read_all(),
            temperatures: sensors::read_all(),
            processes: processes::read_top(18),
            load_average: read_first_fields("/proc/loadavg", 3)
                .unwrap_or_else(|| "unavailable".into()),
            uptime: uptime(),
        }
    }
}

fn read_first_fields(path: &str, count: usize) -> Option<String> {
    let contents = std::fs::read_to_string(path).ok()?;
    Some(
        contents
            .split_whitespace()
            .take(count)
            .collect::<Vec<_>>()
            .join(" "),
    )
}

fn uptime() -> String {
    let seconds = std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|text| text.split_whitespace().next()?.parse::<f64>().ok())
        .map(|seconds| seconds as u64);
    seconds
        .map(|seconds| {
            if seconds >= 86_400 {
                format!("{}d {}h", seconds / 86_400, (seconds % 86_400) / 3600)
            } else {
                format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
            }
        })
        .unwrap_or_else(|| "unknown".into())
}
