use std::collections::HashSet;
use std::fs;
use std::process::Command;
use std::time::Duration;

fn main() {
    let mut args = std::env::args().skip(1);

    match args.next().as_deref() {
        Some("system") => show_system_info(),
        Some("disk") => show_disk_info(),
        Some("memory") => show_memory_info(),
        Some("network") => show_network_info(),
        Some("health") => {
            let json = match args.next().as_deref() {
                None => false,
                Some("--json") => true,
                Some(argument) => {
                    eprintln!("Unknown health option: {argument}\nUse: ruki health [--json]");
                    std::process::exit(2);
                }
            };
            show_health_info(json);
        }
        Some("services") => show_services(),
        Some("processes") => {
            let limit = args
                .next()
                .map(|value| value.parse::<usize>())
                .transpose()
                .unwrap_or_else(|_| {
                    eprintln!("Process limit must be a positive number.");
                    std::process::exit(2);
                })
                .unwrap_or(10);
            if limit == 0 {
                eprintln!("Process limit must be a positive number.");
                std::process::exit(2);
            }
            show_processes(limit);
        }
        Some("help") | Some("--help") | Some("-h") | None => show_help(),
        Some(command) => {
            eprintln!("Unknown command: {command}\n");
            show_help();
            std::process::exit(2);
        }
    }
}

fn show_help() {
    println!("ruki — a terminal-first system management CLI\n");
    println!("Usage: ruki <command> [arguments]");
    println!("       cargo run -- <command>  (from the project folder)\n");
    println!("Commands:");
    println!("  system    Show basic system information");
    println!("  disk      Show disk space for mounted filesystems");
    println!("  memory    Show memory usage");
    println!("  network   Show network interfaces and IP addresses");
    println!("  health    Summarize memory, disk, and network status");
    println!("  services  List active systemd services");
    println!("  processes [number]  Show processes by CPU use (default: 10)");
    println!("  help      Show this help message");
    println!("\nExamples:");
    println!("  cargo run -- help");
    println!("  cargo run -- system");
    println!("  cargo run -- disk");
    println!("  cargo run -- memory");
    println!("  cargo run -- network");
    println!("  cargo run -- health");
    println!("  cargo run -- health --json");
    println!("  cargo run -- services");
    println!("  cargo run -- processes");
    println!("  cargo run -- processes 20");
}

fn show_services() {
    match command_output(
        "systemctl",
        &["list-units", "--type=service", "--state=running", "--no-pager"],
    ) {
        Some(output) => println!("Active systemd services\n{output}"),
        None => {
            eprintln!("Could not list services. Make sure systemd and `systemctl` are available.");
            std::process::exit(1);
        }
    }
}

fn show_network_info() {
    match command_output("ip", &["-brief", "address", "show"]) {
        Some(output) => {
            let active_interfaces = output
                .lines()
                .filter(|line| line.split_whitespace().nth(1) == Some("UP"))
                .collect::<Vec<_>>()
                .join("\n");
            if active_interfaces.is_empty() {
                println!("No active network interfaces found.");
            } else {
                println!("Active network interfaces and addresses\n{active_interfaces}");
            }
        }
        None => {
            eprintln!("Could not show network information. Make sure the `ip` command is available.");
            std::process::exit(1);
        }
    }
}

fn show_processes(limit: usize) {
    match command_output("ps", &["-eo", "pid,comm,%cpu,%mem", "--sort=-%cpu", "--no-headers"]) {
        Some(output) => {
            let top_processes = output.lines().take(limit).collect::<Vec<_>>().join("\n");
            println!("Top {limit} running processes (highest CPU use first)");
            println!("PID COMMAND         %CPU %MEM");
            if !top_processes.is_empty() {
                println!("{top_processes}");
            }
        }
        None => {
            eprintln!("Could not list processes. Make sure the `ps` command is available.");
            std::process::exit(1);
        }
    }
}

fn show_disk_info() {
    match command_output(
        "df",
        &[
            "-h",
            "-l",
            "-x",
            "tmpfs",
            "-x",
            "devtmpfs",
            "-x",
            "efivarfs",
            "-x",
            "squashfs",
            "-x",
            "overlay",
            "--output=source,size,used,avail,pcent,target",
        ],
    ) {
        Some(output) => {
            let mut seen_filesystems = HashSet::new();
            let unique_lines = output
                .lines()
                .filter(|line| {
                    line.split_whitespace()
                        .next()
                        .is_some_and(|filesystem| seen_filesystems.insert(filesystem.to_string()))
                })
                .collect::<Vec<_>>()
                .join("\n");
            println!("Disk space (local storage)\n{unique_lines}");
        }
        None => {
            eprintln!("Could not read disk space. Make sure the `df` command is available.");
            std::process::exit(1);
        }
    }
}

fn show_memory_info() {
    println!("rukiOS memory information");
    println!("Memory: {}", read_memory());
}

fn show_health_info(json: bool) {
    let memory = read_memory();
    let memory_status = memory_status(&memory);
    let disk = root_disk_usage();
    let network = network_status();
    let network_label = if network == "no active interfaces" {
        "WARN"
    } else if network.starts_with("unknown") {
        "UNKNOWN"
    } else {
        "OK"
    };
    let disk_label = disk
        .as_ref()
        .map(|(_, percent)| if *percent >= 85 { "WARN" } else { "OK" })
        .unwrap_or("UNKNOWN");
    let overall_status = overall_health_status(&[memory_status, disk_label, network_label]);

    if json {
        let (disk_status, disk_summary, disk_percent) = match &disk {
            Some((line, percent)) => (
                if *percent >= 85 { "WARN" } else { "OK" },
                format!("\"{}\"", json_escape(line)),
                percent.to_string(),
            ),
            None => ("UNKNOWN", "null".into(), "null".into()),
        };
        println!(
            "{{\"overall_status\":\"{overall_status}\",\"memory\":{{\"status\":\"{memory_status}\",\"summary\":\"{}\"}},\"disk\":{{\"status\":\"{disk_status}\",\"summary\":{disk_summary},\"usage_percent\":{disk_percent}}},\"network\":{{\"status\":\"{network_label}\",\"summary\":\"{}\"}}}}",
            json_escape(&memory),
            json_escape(&network),
        );
        return;
    }

    println!("rukiOS health summary");
    println!("Overall: [{overall_status}]");
    println!("Memory: [{memory_status}] {memory}");

    match disk {
        Some((line, usage_percent)) => {
            let status = if usage_percent >= 85 { "WARN" } else { "OK" };
            println!("Disk: [{status}] {line}");
        }
        None => println!("Disk: [UNKNOWN] could not read root filesystem usage"),
    }

    println!("Network: [{network_label}] {network}");
}

fn overall_health_status(statuses: &[&str]) -> &'static str {
    if statuses.contains(&"WARN") {
        "WARN"
    } else if statuses.contains(&"UNKNOWN") {
        "UNKNOWN"
    } else {
        "OK"
    }
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character.is_control() => {
                escaped.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }
    escaped
}

fn root_disk_usage() -> Option<(String, u64)> {
    let output = command_output(
        "df",
        &["-h", "--output=source,size,used,avail,pcent,target", "/"],
    )?;
    let line = output.lines().nth(1)?.trim().to_string();
    let usage_percent = line
        .split_whitespace()
        .nth(4)?
        .trim_end_matches('%')
        .parse::<u64>()
        .ok()?;
    Some((line, usage_percent))
}

fn memory_status(memory: &str) -> &'static str {
    let values = memory.split_whitespace().collect::<Vec<_>>();
    let used = values.first().and_then(|value| value.parse::<u64>().ok());
    let total = values.get(4).and_then(|value| value.parse::<u64>().ok());

    match (used, total) {
        (Some(used), Some(total)) if total > 0 && used.saturating_mul(100) / total >= 90 => "WARN",
        (Some(_), Some(total)) if total > 0 => "OK",
        _ => "UNKNOWN",
    }
}

fn network_status() -> String {
    match command_output("ip", &["-brief", "address", "show"]) {
        Some(output) => {
            let active_count = output
                .lines()
                .filter(|line| line.split_whitespace().nth(1) == Some("UP"))
                .count();
            if active_count == 0 {
                "no active interfaces".into()
            } else {
                format!("{active_count} active interface(s)")
            }
        }
        None => "unknown (`ip` command unavailable)".into(),
    }
}

fn show_system_info() {
    let os = read_os_name();
    let kernel = command_output("uname", &["-r"]).unwrap_or_else(|| "unknown".into());
    let uptime = read_uptime();
    let memory = read_memory();

    println!("rukiOS system information");
    println!("OS:      {os}");
    println!("Kernel:  {kernel}");
    println!("Uptime:  {uptime}");
    println!("Memory:  {memory}");
}

fn read_os_name() -> String {
    let contents = fs::read_to_string("/etc/os-release").unwrap_or_default();
    contents
        .lines()
        .find_map(|line| line.strip_prefix("PRETTY_NAME=\""))
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or("Unknown Linux distribution")
        .to_string()
}

fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn read_uptime() -> String {
    let seconds = fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|contents| contents.split_whitespace().next()?.parse::<f64>().ok())
        .map(|value| value as u64);

    match seconds {
        Some(seconds) => {
            let duration = Duration::from_secs(seconds);
            let days = duration.as_secs() / 86_400;
            let hours = (duration.as_secs() % 86_400) / 3_600;
            let minutes = (duration.as_secs() % 3_600) / 60;
            if days > 0 {
                format!("{days}d {hours}h {minutes}m")
            } else if hours > 0 {
                format!("{hours}h {minutes}m")
            } else {
                format!("{minutes}m")
            }
        }
        None => "unknown".into(),
    }
}

fn read_memory() -> String {
    let contents = match fs::read_to_string("/proc/meminfo") {
        Ok(contents) => contents,
        Err(_) => return "unknown".into(),
    };

    let mut total_kib = None;
    let mut available_kib = None;
    for line in contents.lines() {
        if let Some(value) = line.strip_prefix("MemTotal:") {
            total_kib = value.split_whitespace().next().and_then(|n| n.parse::<u64>().ok());
        } else if let Some(value) = line.strip_prefix("MemAvailable:") {
            available_kib = value.split_whitespace().next().and_then(|n| n.parse::<u64>().ok());
        }
    }

    match (total_kib, available_kib) {
        (Some(total), Some(available)) => {
            let used = total.saturating_sub(available);
            format!("{} MiB used / {} MiB total", used / 1024, total / 1024)
        }
        _ => "unknown".into(),
    }
}
