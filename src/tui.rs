use std::io;
use std::process::Command;
use std::collections::HashSet;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
};

const SECTIONS: [&str; 8] = [
    "Overview",
    "System",
    "Disk",
    "Memory",
    "Network",
    "Processes",
    "Services",
    "Health",
];

struct App {
    selected: usize,
    scroll: u16,
    snapshot: Snapshot,
}

impl Default for App {
    fn default() -> Self {
        Self {
            selected: 0,
            scroll: 0,
            snapshot: Snapshot::load(),
        }
    }
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('l') | KeyCode::Right => self.select_next(),
                    KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('h') | KeyCode::Left => self.select_previous(),
                    KeyCode::PageDown => self.scroll = self.scroll.saturating_add(8),
                    KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(8),
                    KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.scroll = self.scroll.saturating_add(8);
                    }
                    KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.scroll = self.scroll.saturating_sub(8);
                    }
                    KeyCode::Char('r') => {
                        self.snapshot = Snapshot::load();
                        self.scroll = 0;
                    }
                    KeyCode::Home => self.scroll = 0,
                    KeyCode::Char('g') => {
                        self.selected = 0;
                        self.scroll = 0;
                    }
                    KeyCode::Char('G') => {
                        self.selected = SECTIONS.len() - 1;
                        self.scroll = 0;
                    }
                    KeyCode::Char(number @ '1'..='8') => {
                        self.selected = number.to_digit(10).unwrap_or(1) as usize - 1;
                        self.scroll = 0;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn select_next(&mut self) {
        self.selected = (self.selected + 1) % SECTIONS.len();
        self.scroll = 0;
    }

    fn select_previous(&mut self) {
        self.selected = self.selected.checked_sub(1).unwrap_or(SECTIONS.len() - 1);
        self.scroll = 0;
    }

    fn draw(&self, frame: &mut Frame) {
        let page = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(4)])
            .split(frame.area());

        let header = Paragraph::new(Line::from(vec![
            Span::styled(" rukiOS ", Style::default().fg(Color::Black).bg(Color::Cyan).bold()),
            Span::styled("  SYSTEM CONSOLE", Style::default().fg(Color::White).bold()),
            Span::styled("                                      RUST • LINUX", Style::default().fg(Color::Gray)),
        ]))
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
        frame.render_widget(header, page[0]);

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(22), Constraint::Min(0)])
            .split(page[1]);
        self.draw_navigation(frame, body[0]);
        if self.selected == 0 {
            self.draw_overview(frame, body[1]);
        } else {
            self.draw_section(frame, body[1]);
        }

        let footer = Paragraph::new(vec![
            Line::from(vec![
                keycap("j/k"), key_label(" move  "),
                keycap("h/l"), key_label(" prev/next  "),
                keycap("←/→"), key_label(" prev/next  "),
                keycap("1-8"), key_label(" jump to section"),
            ]),
            Line::from(vec![
                keycap("g/G"), key_label(" first/last section  "),
                keycap("r"), key_label(" refresh  "),
                keycap("Ctrl-u/d"), key_label(" scroll  "),
                keycap("q/Esc"), key_label(" quit"),
            ]),
        ])
        .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(Color::DarkGray)));
        frame.render_widget(footer, page[2]);
    }

    fn draw_navigation(&self, frame: &mut Frame, area: Rect) {
        let items = SECTIONS
            .iter()
            .enumerate()
            .map(|(index, title)| {
                let label = format!(" {}  {}", index + 1, title);
                let style = if index == self.selected {
                    Style::default().fg(Color::Black).bg(Color::Cyan).bold()
                } else {
                    Style::default().fg(Color::Gray)
                };
                ListItem::new(label).style(style)
            })
            .collect::<Vec<_>>();
        let nav = List::new(items)
            .block(
                Block::default()
                    .title(" SECTIONS ")
                    .borders(Borders::RIGHT)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .highlight_symbol("› ");
        frame.render_widget(nav, area);
    }

    fn draw_section(&self, frame: &mut Frame, area: Rect) {
        let title = SECTIONS[self.selected];
        let content = self.snapshot.section_content(self.selected);
        let paragraph = Paragraph::new(content)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .title(format!(" {title} "))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));
        frame.render_widget(paragraph, area);
    }

    fn draw_overview(&self, frame: &mut Frame, area: Rect) {
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(5)])
            .margin(1)
            .split(area);

        let memory = &self.snapshot.memory;
        let memory_percent = memory_percent(&memory);
        let disk = &self.snapshot.disk;
        let disk_percent = disk.as_ref().map(|(_, percent)| *percent).unwrap_or(0);
        let active_interfaces = self.snapshot.active_interfaces;
        let overall = health_status(memory_percent, disk.as_ref().map(|(_, p)| *p), active_interfaces);

        let title = Paragraph::new(Line::from(vec![
            Span::styled("SYSTEM OVERVIEW  ", Style::default().fg(Color::Cyan).bold()),
            Span::styled(format!("Overall: {overall}"), status_style(overall)),
        ]));
        frame.render_widget(title, sections[0]);

        let meters = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(sections[1]);
        let memory_gauge = Gauge::default()
            .block(Block::default().title(" MEMORY ").borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)))
            .gauge_style(Style::default().fg(if memory_percent >= 90 { Color::Red } else { Color::Blue }))
            .ratio(f64::from(memory_percent.min(100)) / 100.0)
            .label(format!("{memory_percent}% used"));
        frame.render_widget(memory_gauge, meters[0]);

        let disk_gauge = Gauge::default()
            .block(Block::default().title(" ROOT DISK ").borders(Borders::ALL).border_style(Style::default().fg(if disk_percent >= 85 { Color::Yellow } else { Color::Green })))
            .gauge_style(Style::default().fg(if disk_percent >= 85 { Color::Yellow } else { Color::Green }))
            .ratio(f64::from(disk_percent.min(100)) / 100.0)
            .label(format!("{disk_percent}% used"));
        frame.render_widget(disk_gauge, meters[1]);

        let mut content = vec![
            Line::from(Span::styled("QUICK STATUS", Style::default().fg(Color::Cyan).bold())),
            Line::from(format!("  Network        {active_interfaces} active interfaces")),
            Line::from(format!("  Root disk      {}", disk.as_ref().map(|(line, _)| line.as_str()).unwrap_or("unavailable"))),
            Line::from(format!("  Memory         {memory}")),
            Line::from(""),
            Line::from(Span::styled("SYSTEM", Style::default().fg(Color::Cyan).bold())),
        ];
        content.extend(self.snapshot.system.lines().map(|line| Line::from(format!("  {line}"))));
        content.push(Line::from(""));
        content.push(Line::from(Span::styled(
            "Choose a section on the left to inspect details.",
            Style::default().fg(Color::Gray),
        )));
        let details = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)))
            .wrap(Wrap { trim: false });
        frame.render_widget(details, sections[2]);
    }
}

struct Snapshot {
    system: String,
    disk_listing: String,
    disk: Option<(String, u16)>,
    memory: String,
    network_listing: String,
    active_interfaces: usize,
    processes: String,
    services: String,
    health: String,
}

impl Snapshot {
    fn load() -> Self {
        let memory = memory_reading();
        let disk = root_disk_reading();
        let network_listing = network_listing();
        let active_interfaces = active_network_count();
        let system = system_summary(&memory);
        let disk_listing = disk_listing();
        let processes = process_listing();
        let services = service_listing();
        let health = health_summary(&memory, disk.as_ref(), active_interfaces);

        Self {
            system,
            disk_listing,
            disk,
            memory,
            network_listing,
            active_interfaces,
            processes,
            services,
            health,
        }
    }

    fn section_content(&self, section: usize) -> String {
        match section {
            1 => self.system.clone(),
            2 => self.disk_listing.clone(),
            3 => format!("Memory usage\n\n{}", self.memory),
            4 => self.network_listing.clone(),
            5 => self.processes.clone(),
            6 => self.services.clone(),
            7 => self.health.clone(),
            _ => String::new(),
        }
    }
}

pub fn run() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}

fn keycap(key: &'static str) -> Span<'static> {
    Span::styled(
        format!(" {key} "),
        Style::default()
            .fg(Color::Cyan)
            .bg(Color::Rgb(38, 50, 64))
            .add_modifier(Modifier::BOLD),
    )
}

fn key_label(label: &'static str) -> Span<'static> {
    Span::styled(label, Style::default().fg(Color::Gray))
}

fn system_summary(memory: &str) -> String {
    let os = std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|text| text.lines().find_map(|line| line.strip_prefix("PRETTY_NAME=\"").and_then(|s| s.strip_suffix('"'))).map(str::to_owned))
        .unwrap_or_else(|| "Unknown Linux distribution".into());
    let kernel = run_command("uname", &["-r"]).unwrap_or_else(|| "unknown".into());
    let uptime = std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse::<f64>().ok())
        .map(|seconds| format!("{} hours, {} minutes", seconds as u64 / 3600, (seconds as u64 % 3600) / 60))
        .unwrap_or_else(|| "unknown".into());
    format!("Operating system  {os}\nKernel            {kernel}\nUptime            {uptime}\nMemory            {memory}")
}

fn disk_listing() -> String {
    let Some(output) = run_command("df", &["-h", "-l", "-x", "tmpfs", "-x", "devtmpfs", "-x", "efivarfs", "-x", "squashfs", "-x", "overlay", "--output=source,size,used,avail,pcent,target"]) else {
        return "Could not read disk information (is `df` installed?)".into();
    };
    let mut seen = HashSet::new();
    output
        .lines()
        .filter(|line| line.split_whitespace().next().is_some_and(|source| seen.insert(source.to_string())))
        .collect::<Vec<_>>()
        .join("\n")
}

fn memory_reading() -> String {
    let contents = match std::fs::read_to_string("/proc/meminfo") {
        Ok(contents) => contents,
        Err(_) => return "Memory information unavailable".into(),
    };
    let mut total = None;
    let mut available = None;
    for line in contents.lines() {
        if let Some(value) = line.strip_prefix("MemTotal:") {
            total = value.split_whitespace().next().and_then(|n| n.parse::<u64>().ok());
        } else if let Some(value) = line.strip_prefix("MemAvailable:") {
            available = value.split_whitespace().next().and_then(|n| n.parse::<u64>().ok());
        }
    }
    match (total, available) {
        (Some(total), Some(available)) => format!("{} MiB used / {} MiB total", total.saturating_sub(available) / 1024, total / 1024),
        _ => "Memory information unavailable".into(),
    }
}

fn memory_percent(memory: &str) -> u16 {
    let values = memory.split_whitespace().collect::<Vec<_>>();
    let used = values.first().and_then(|n| n.parse::<u64>().ok());
    let total = values.get(4).and_then(|n| n.parse::<u64>().ok());
    match (used, total) {
        (Some(used), Some(total)) if total > 0 => ((used.saturating_mul(100) / total).min(100)) as u16,
        _ => 0,
    }
}

fn root_disk_reading() -> Option<(String, u16)> {
    let output = run_command("df", &["-h", "--output=source,size,used,avail,pcent,target", "/"])?;
    let line = output.lines().nth(1)?.trim().to_string();
    let percent = line.split_whitespace().nth(4)?.trim_end_matches('%').parse::<u16>().ok()?;
    Some((line, percent))
}

fn network_listing() -> String {
    run_command("ip", &["-brief", "address", "show"])
        .map(|output| output.lines().filter(|line| line.split_whitespace().nth(1) == Some("UP")).collect::<Vec<_>>().join("\n"))
        .filter(|output| !output.is_empty())
        .unwrap_or_else(|| "No active network interfaces found.".into())
}

fn active_network_count() -> usize {
    run_command("ip", &["-brief", "address", "show"])
        .map(|output| output.lines().filter(|line| line.split_whitespace().nth(1) == Some("UP")).count())
        .unwrap_or(0)
}

fn process_listing() -> String {
    run_command("ps", &["-eo", "pid,comm,%cpu,%mem", "--sort=-%cpu", "--no-headers"])
        .map(|output| {
            let rows = output.lines().take(20).collect::<Vec<_>>().join("\n");
            format!("PID COMMAND         %CPU %MEM\n{rows}")
        })
        .unwrap_or_else(|| "Could not list processes (is `ps` installed?)".into())
}

fn service_listing() -> String {
    run_command("systemctl", &["list-units", "--type=service", "--state=running", "--no-pager", "--no-legend"])
        .unwrap_or_else(|| "Could not list systemd services.".into())
}

fn health_summary(memory: &str, disk: Option<&(String, u16)>, interfaces: usize) -> String {
    let memory_percent = memory_percent(&memory);
    let disk_percent = disk.map(|(_, percent)| *percent);
    let overall = health_status(memory_percent, disk_percent, interfaces);
    format!(
        "Overall     {overall}\nMemory      {}  {memory}\nDisk        {}  {}\nNetwork     {} active interface(s)",
        if memory_percent >= 90 { "WARN" } else { "OK" },
        disk_percent.map(|p| if p >= 85 { "WARN" } else { "OK" }).unwrap_or("UNKNOWN"),
        disk.map(|(line, _)| line.as_str()).unwrap_or("unavailable"),
        if interfaces == 0 { "WARN" } else { "OK" },
    )
}

fn health_status(memory_percent: u16, disk_percent: Option<u16>, interfaces: usize) -> &'static str {
    if memory_percent >= 90 || disk_percent.is_some_and(|percent| percent >= 85) || interfaces == 0 {
        "WARN"
    } else if disk_percent.is_none() {
        "UNKNOWN"
    } else {
        "OK"
    }
}

fn status_style(status: &str) -> Style {
    match status {
        "WARN" => Style::default().fg(Color::Red).bold(),
        "UNKNOWN" => Style::default().fg(Color::Yellow).bold(),
        _ => Style::default().fg(Color::Green).bold(),
    }
}

fn run_command(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    output.status.success().then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}
