use std::process::Command;

#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub user: String,
    pub name: String,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub depth: usize,
}

pub(super) fn read_top(limit: usize) -> Vec<ProcessInfo> {
    let Ok(output) = Command::new("ps")
        .args(["-eo", "pid=,ppid=,user=,comm=,pcpu=,pmem=", "--sort=-pcpu"])
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut all = Vec::new();
    for line in text.lines() {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        let Some((pid, ppid, user, name, cpu, memory)) = parse(&fields) else {
            continue;
        };
        all.push(ProcessInfo {
            pid,
            ppid,
            user,
            name,
            cpu_percent: cpu,
            memory_percent: memory,
            depth: 0,
        });
    }
    let parents = all
        .iter()
        .map(|process| (process.pid, process.ppid))
        .collect::<std::collections::HashMap<_, _>>();
    all.sort_by(|left, right| right.cpu_percent.total_cmp(&left.cpu_percent));
    all.truncate(limit);
    for process in &mut all {
        let mut parent = process.ppid;
        let mut depth = 0;
        while depth < 4 && parent != 0 {
            let Some(next) = parents.get(&parent) else {
                break;
            };
            depth += 1;
            parent = *next;
        }
        process.depth = depth;
    }
    all
}

fn parse(fields: &[&str]) -> Option<(u32, u32, String, String, f64, f64)> {
    Some((
        fields.first()?.parse().ok()?,
        fields.get(1)?.parse().ok()?,
        fields.get(2)?.to_string(),
        fields.get(3)?.to_string(),
        fields.get(4)?.parse().ok()?,
        fields.get(5)?.parse().ok()?,
    ))
}
