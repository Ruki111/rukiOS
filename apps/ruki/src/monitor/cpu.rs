use std::{collections::HashMap, fs};

pub(super) fn read_counters() -> HashMap<String, (u64, u64)> {
    let Ok(contents) = fs::read_to_string("/proc/stat") else {
        return HashMap::new();
    };
    parse_counters(&contents)
}

fn parse_counters(contents: &str) -> HashMap<String, (u64, u64)> {
    contents
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let name = fields.next()?;
            if name != "cpu" && !is_cpu_core(name) {
                return None;
            }
            let counters = fields
                .map(str::parse::<u64>)
                .collect::<Result<Vec<_>, _>>()
                .ok()?;
            // guest and guest_nice are already included in user/nice on Linux.
            let total = counters.iter().take(8).copied().sum();
            let idle =
                counters.get(3).copied().unwrap_or(0) + counters.get(4).copied().unwrap_or(0);
            Some((name.to_owned(), (total, idle)))
        })
        .collect()
}

fn is_cpu_core(name: &str) -> bool {
    name.strip_prefix("cpu")
        .is_some_and(|suffix| !suffix.is_empty() && suffix.chars().all(|ch| ch.is_ascii_digit()))
}

pub(super) fn usage(previous: &(u64, u64), current: &(u64, u64)) -> Option<f64> {
    let total_delta = current.0.checked_sub(previous.0)?;
    let idle_delta = current.1.checked_sub(previous.1)?;
    if total_delta == 0 {
        return None;
    }
    Some(total_delta.saturating_sub(idle_delta) as f64 / total_delta as f64 * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_total_and_per_core_cpu_counters() {
        let counters =
            parse_counters("cpu 100 20 30 40 10 0 0 0 9 9\ncpu0 50 5 10 20 5 0 0 0\nbtime 123\n");

        assert_eq!(counters.get("cpu"), Some(&(200, 50)));
        assert_eq!(counters.get("cpu0"), Some(&(90, 25)));
        assert_eq!(counters.len(), 2);
    }

    #[test]
    fn ignores_malformed_cpu_lines() {
        let counters = parse_counters("cpu not-a-number\ncpuX 1 2 3\ncpu1 1 2 invalid 4\n");
        assert!(counters.is_empty());
    }

    #[test]
    fn calculates_usage_and_handles_zero_or_reset_counters() {
        assert_eq!(usage(&(100, 40), &(160, 70)), Some(50.0));
        assert_eq!(usage(&(10, 5), &(10, 5)), None);
        assert_eq!(usage(&(100, 40), &(90, 35)), None);
    }
}
