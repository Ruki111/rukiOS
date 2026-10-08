use std::{collections::HashMap, fs, process::Command};

#[derive(Clone, Debug)]
pub struct RootDisk {
    pub device: String,
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_percent: u16,
}

#[derive(Clone, Debug)]
pub struct DiskIo {
    pub name: String,
    pub read_bytes_per_sec: Option<u64>,
    pub write_bytes_per_sec: Option<u64>,
}

pub(super) fn read_root() -> Option<RootDisk> {
    let output = Command::new("df")
        .args(["-B1", "--output=source,size,used,avail,pcent", "/"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let fields = text.lines().nth(1)?.split_whitespace().collect::<Vec<_>>();
    let used_percent = fields.get(4)?.trim_end_matches('%').parse().ok()?;
    Some(RootDisk {
        device: fields.first()?.to_string(),
        total_bytes: fields.get(1)?.parse().ok()?,
        used_bytes: fields.get(2)?.parse().ok()?,
        available_bytes: fields.get(3)?.parse().ok()?,
        used_percent,
    })
}

pub(super) fn read_io_counters() -> HashMap<String, (u64, u64)> {
    let Ok(contents) = fs::read_to_string("/proc/diskstats") else {
        return HashMap::new();
    };
    contents
        .lines()
        .filter_map(|line| {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            let name = *fields.get(2)?;
            if !is_disk(name)
                || std::path::Path::new(&format!("/sys/class/block/{name}/partition")).exists()
            {
                return None;
            }
            // Linux reports sectors in 512-byte units in /proc/diskstats.
            let sectors_read = fields.get(5)?.parse::<u64>().ok()?;
            let sectors_written = fields.get(9)?.parse::<u64>().ok()?;
            Some((
                name.to_owned(),
                (
                    sectors_read.saturating_mul(512),
                    sectors_written.saturating_mul(512),
                ),
            ))
        })
        .collect()
}

fn is_disk(name: &str) -> bool {
    ["sd", "hd", "vd", "xvd", "nvme", "mmcblk"]
        .iter()
        .any(|prefix| name.starts_with(prefix))
}
