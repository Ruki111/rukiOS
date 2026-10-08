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
