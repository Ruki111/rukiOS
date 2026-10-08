use std::fs;

#[derive(Clone, Copy, Debug)]
pub struct MemoryMetrics {
    pub used_kib: u64,
    pub total_kib: u64,
    pub swap_used_kib: u64,
    pub swap_total_kib: u64,
}

pub(super) fn read() -> Option<MemoryMetrics> {
    let contents = fs::read_to_string("/proc/meminfo").ok()?;
    parse(&contents)
}

fn parse(contents: &str) -> Option<MemoryMetrics> {
    let value = |key: &str| -> Option<u64> {
        contents.lines().find_map(|line| {
            line.strip_prefix(key)?
                .split_whitespace()
                .next()?
                .parse()
                .ok()
        })
    };
    let total_kib = value("MemTotal:")?;
    let available_kib = value("MemAvailable:")?;
    let swap_total_kib = value("SwapTotal:").unwrap_or(0);
    let swap_free_kib = value("SwapFree:").unwrap_or(0);
    Some(MemoryMetrics {
        used_kib: total_kib.saturating_sub(available_kib),
        total_kib,
        swap_used_kib: swap_total_kib.saturating_sub(swap_free_kib),
        swap_total_kib,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_ram_and_swap_usage() {
        let metrics = parse(
            "MemTotal:       1000 kB\nMemAvailable:    250 kB\nSwapTotal:       500 kB\nSwapFree:        125 kB\n",
        )
        .unwrap();

        assert_eq!(metrics.total_kib, 1000);
        assert_eq!(metrics.used_kib, 750);
        assert_eq!(metrics.swap_total_kib, 500);
        assert_eq!(metrics.swap_used_kib, 375);
    }

    #[test]
    fn missing_required_memory_values_returns_none() {
        assert!(parse("MemTotal: 1000 kB\n").is_none());
    }

    #[test]
    fn missing_swap_values_are_reported_as_zero() {
        let metrics = parse("MemTotal: 1000 kB\nMemAvailable: 1000 kB\n").unwrap();
        assert_eq!(metrics.swap_total_kib, 0);
        assert_eq!(metrics.swap_used_kib, 0);
    }
}
