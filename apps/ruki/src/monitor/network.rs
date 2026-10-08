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
    contents.lines().skip(2).filter_map(parse_line).collect()
}

fn parse_line(line: &str) -> Option<NetworkMetric> {
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
}

fn is_physical_candidate(name: &str) -> bool {
    (name.starts_with("en") || name.starts_with("wl") || name.starts_with("eth")) && name != "lo"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_receive_and_transmit_bytes_for_physical_interfaces() {
        let metric = parse_line("enp2s0: 100 1 2 3 4 5 6 7 900 8 9 10 11 12 13 14").unwrap();
        assert_eq!(metric.name, "enp2s0");
        assert_eq!(metric.rx_bytes, 100);
        assert_eq!(metric.tx_bytes, 900);
        assert_eq!(metric.rx_bytes_per_sec, None);
    }

    #[test]
    fn ignores_loopback_and_virtual_interfaces() {
        assert!(parse_line("lo: 100 0 0 0 0 0 0 0 100 0 0 0 0 0 0 0").is_none());
        assert!(parse_line("docker0: 100 0 0 0 0 0 0 0 100 0 0 0 0 0 0 0").is_none());
    }

    #[test]
    fn malformed_network_counters_are_ignored() {
        assert!(parse_line("wlan0: 10 2 invalid").is_none());
        assert!(parse_line("wlan0 without delimiter").is_none());
    }
}
