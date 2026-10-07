use std::fs;
use std::process::Command;
use std::time::Duration;

fn main() {
    let mut args = std::env::args().skip(1);

    match args.next().as_deref() {
        Some("system") => show_system_info(),
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
    println!("Usage: ruki <command>\n");
    println!("Commands:");
    println!("  system    Show basic system information");
    println!("  help      Show this help message");
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
