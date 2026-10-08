use std::fs;

#[derive(Clone, Debug)]
pub struct NetworkMetric {
    pub name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_bytes_per_sec: Option<u64>,
    pub tx_bytes_per_sec: Option<u64>,
}

pub(super) fn read_counters() -> Vec<NetworkMetric> {
    let Ok(contents) = fs::read_to_string("/proc/net/dev") else {
        return Vec::new();
    };
    contents
        .lines()
        .skip(2)
        .filter_map(|line| {
            let (name, counters) = line.split_once(':')?;
            let name = name.trim();
            if !is_physical_candidate(name) {
                return None;
            }
            let counters = counters
                .split_whitespace()
                .map(str::parse::<u64>)
                .collect::<Result<Vec<_>, _>>()
                .ok()?;
            Some(NetworkMetric {
                name: name.to_owned(),
                rx_bytes: *counters.first()?,
                tx_bytes: *counters.get(8)?,
                rx_bytes_per_sec: None,
                tx_bytes_per_sec: None,
            })
        })
        .collect()
}

fn is_physical_candidate(name: &str) -> bool {
    (name.starts_with("en") || name.starts_with("wl") || name.starts_with("eth")) && name != "lo"
}
