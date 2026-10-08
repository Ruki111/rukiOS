use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEventKind,
    },
    execute,
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Sparkline, Table, Wrap},
};

use crate::monitor::{Metrics, Sampler};

const BG: Color = Color::Rgb(25, 26, 25);
const FG: Color = Color::Rgb(218, 215, 190);
const MUTED: Color = Color::Rgb(125, 135, 116);
const ACCENT: Color = Color::Rgb(190, 185, 54);
const GREEN: Color = Color::Rgb(121, 166, 80);
const RED: Color = Color::Rgb(202, 92, 92);

pub fn run() -> io::Result<()> {
    ratatui::run(|terminal| {
        execute!(terminal.backend_mut(), EnableMouseCapture)?;
        let result = App::new().run(terminal);
        execute!(terminal.backend_mut(), DisableMouseCapture)?;
        result
    })
}

struct App {
    sampler: Sampler,
    metrics: Metrics,
    last_sample: Instant,
    process_offset: usize,
    process_visible_rows: usize,
    selected_panel: usize,
}

impl App {
    fn new() -> Self {
        let mut sampler = Sampler::new();
        let metrics = sampler.sample();
        Self {
            sampler,
            metrics,
            last_sample: Instant::now(),
            process_offset: 0,
            process_visible_rows: 1,
            selected_panel: 0,
        }
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            if self.last_sample.elapsed() >= Duration::from_secs(1) {
                self.metrics = self.sampler.sample();
                self.last_sample = Instant::now();
                self.process_offset = self.process_offset.min(self.process_max_offset());
            }
            terminal.draw(|frame| self.draw(frame))?;
            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('r') => {
                            self.metrics = self.sampler.sample();
                            self.last_sample = Instant::now();
                        }
                        KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                            self.selected_panel = (self.selected_panel + 1) % 5
                        }
                        KeyCode::Char(number @ '1'..='5') => {
                            self.selected_panel = number.to_digit(10).unwrap_or(1) as usize - 1;
                        }
                        KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
                            self.selected_panel = self.selected_panel.checked_sub(1).unwrap_or(4)
                        }
                        KeyCode::Down | KeyCode::Char('j') if self.selected_panel == 4 => {
                            self.process_offset =
                                (self.process_offset + 1).min(self.process_max_offset());
                        }
                        KeyCode::Up | KeyCode::Char('k') if self.selected_panel == 4 => {
                            self.process_offset = self.process_offset.saturating_sub(1)
                        }
                        KeyCode::PageDown if self.selected_panel == 4 => {
                            self.scroll_processes(self.process_visible_rows as isize);
                        }
                        KeyCode::PageUp if self.selected_panel == 4 => {
                            self.scroll_processes(-(self.process_visible_rows as isize));
                        }
                        KeyCode::Home | KeyCode::Char('g') if self.selected_panel == 4 => {
                            self.process_offset = 0;
                        }
                        KeyCode::End | KeyCode::Char('G') if self.selected_panel == 4 => {
                            self.process_offset = self.process_max_offset();
                        }
                        _ => {}
                    },
                    Event::Mouse(mouse)
                        if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) =>
                    {
                        self.select_tab_at(mouse.column, mouse.row);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(
            Block::default().style(Style::default().bg(BG)),
            frame.area(),
        );
        let area = frame.area();
        if area.width < 80 || area.height < 24 {
            let message = Paragraph::new(vec![
                Line::from(Span::styled(
                    "rukiOS SYSTEM MONITOR",
                    Style::default().fg(ACCENT).bold(),
                )),
                Line::from("Terminal is too small for the dashboard."),
                Line::from("Resize to at least 80 columns by 24 rows."),
                Line::from("Press q or Escape to quit."),
            ])
            .style(Style::default().fg(FG))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(MUTED)),
            );
            frame.render_widget(message, area);
            return;
        }
        let page = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(8),
                Constraint::Min(8),
                Constraint::Length(2),
            ])
            .margin(1)
            .split(frame.area());
        self.draw_tabs(frame, page[0]);
        self.draw_cpu(frame, page[1]);
        let lower = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(41), Constraint::Percentage(59)])
            .split(page[2]);
        self.process_visible_rows = lower[1].height.saturating_sub(3).max(1) as usize;
        self.process_offset = self.process_offset.min(self.process_max_offset());
        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(37),
                Constraint::Percentage(31),
                Constraint::Percentage(32),
            ])
            .split(lower[0]);
        self.draw_memory(frame, left[0]);
        self.draw_disks(frame, left[1]);
        self.draw_network(frame, left[2]);
        self.draw_processes(frame, lower[1]);
        let footer = Paragraph::new(vec![
            Line::from(vec![
                key("Tab/h/l"),
                hint(" tabs  "),
                key("1-5"),
                hint(" jump  "),
                key("j/k/↑↓"),
                hint(" rows  "),
                key("PgUp/Dn"),
                hint(" page"),
            ]),
            Line::from(vec![
                key("g/G"),
                hint(" first/last page  "),
                key("click"),
                hint(" tabs  "),
                key("r"),
                hint(" refresh  "),
                key("q/Esc"),
                hint(" quit"),
            ]),
        ])
        .style(Style::default().fg(FG));
        frame.render_widget(footer, page[3]);
    }

    fn draw_tabs(&self, frame: &mut Frame, area: Rect) {
        let names = ["CPU", "Memory", "Disks", "Network", "Processes"];
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);
        let heading = Paragraph::new(Line::from(vec![
            Span::styled(
                " rukiOS ",
                Style::default()
                    .fg(BG)
                    .bg(ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" SYSTEM MONITOR", Style::default().fg(FG).bold()),
            Span::styled(
                format!(
                    "     UP {}   LOAD {}",
                    self.metrics.uptime, self.metrics.load_average
                ),
                Style::default().fg(MUTED),
            ),
        ]));
        frame.render_widget(heading, rows[0]);
        let mut spans = Vec::new();
        for (i, name) in names.iter().enumerate() {
            let selected = self.selected_panel == i;
            spans.push(Span::styled(
                format!(" [{} {}] ", i + 1, name),
                if selected {
                    Style::default().fg(BG).bg(ACCENT).bold()
                } else {
                    Style::default().fg(FG)
                },
            ));
        }
        frame.render_widget(
            Paragraph::new(Line::from(spans)).block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(MUTED)),
            ),
            rows[1],
        );
    }

    fn process_max_offset(&self) -> usize {
        self.metrics
            .processes
            .len()
            .saturating_sub(self.process_visible_rows.max(1))
    }

    fn scroll_processes(&mut self, amount: isize) {
        let next = self.process_offset as isize + amount;
        self.process_offset = next.clamp(0, self.process_max_offset() as isize) as usize;
    }

    fn select_tab_at(&mut self, x: u16, y: u16) {
        let mut start = 1u16;
        let names = ["CPU", "Memory", "Disks", "Network", "Processes"];
        for (index, name) in names.iter().enumerate() {
            let width = name.len() as u16 + 6; // brackets, digit, and surrounding spaces
            if y == 2 && x >= start && x < start + width {
                self.selected_panel = index;
                return;
            }
            start = start.saturating_add(width);
        }
    }

    fn draw_cpu(&self, frame: &mut Frame, area: Rect) {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(66), Constraint::Percentage(34)])
            .split(area);
        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(1)])
            .margin(1)
            .split(split[0]);
        let percent = self
            .metrics
            .cpu_percent
            .map(|value| format!("{value:.0}%"))
            .unwrap_or_else(|| "N/A".into());
        let top = Paragraph::new(Line::from(vec![
            Span::styled("CPU ", Style::default().fg(ACCENT).bold()),
            Span::styled(percent.clone(), Style::default().fg(FG).bold()),
            Span::styled("   per-core utilization", Style::default().fg(MUTED)),
        ]));
        frame.render_widget(top, left[0]);
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(left[1]);
        for (column, cores) in columns.iter().zip(
            self.metrics
                .cores
                .chunks(self.metrics.cores.len().div_ceil(2).max(1)),
        ) {
            let lines = cores
                .iter()
                .map(|(name, use_pct)| {
                    let value = use_pct
                        .map(|n| format!("{n:>3.0}%"))
                        .unwrap_or_else(|| " N/A".into());
                    let bar = use_pct.map(|n| bar(n, 9)).unwrap_or_default();
                    Line::from(vec![
                        Span::styled(format!("{name:<5}"), Style::default().fg(MUTED)),
                        Span::styled(
                            bar,
                            Style::default().fg(usage_color(use_pct.unwrap_or(0.0))),
                        ),
                        Span::styled(format!(" {value}"), Style::default().fg(FG)),
                    ])
                })
                .collect::<Vec<_>>();
            frame.render_widget(Paragraph::new(lines), *column);
        }
        let gpu = self
            .metrics
            .gpus
            .iter()
            .map(|g| {
                let usage = g
                    .utilization_percent
                    .map(|v| format!("{v}%"))
                    .unwrap_or_else(|| "N/A".into());
                let temp = g
                    .temperature_celsius
                    .map(|v| format!("{v:.0}°C"))
                    .unwrap_or_else(|| "N/A".into());
                let vram = match (g.memory_used_bytes, g.memory_total_bytes) {
                    (Some(used), Some(total)) => format!("{} / {}", bytes(used), bytes(total)),
                    _ => "VRAM N/A".into(),
                };
                Line::from(format!("{}  {usage}  {vram}  {temp}", g.name))
            })
            .collect::<Vec<_>>();
        let mut right_lines = vec![Line::from(Span::styled(
            "GPU / SENSORS",
            Style::default().fg(ACCENT).bold(),
        ))];
        let right_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(2), Constraint::Length(3)])
            .margin(1)
            .split(split[1]);
        let gpu_lines = if gpu.is_empty() {
            vec![Line::from("GPU metrics unavailable")]
        } else {
            gpu
        };
        right_lines.extend(gpu_lines);
        right_lines.extend(
            self.metrics
                .temperatures
                .iter()
                .take(2)
                .map(|sensor| Line::from(format!("{}  {:.0}°C", sensor.label, sensor.celsius))),
        );
        right_lines.push(Line::from(Span::styled(
            "CPU HISTORY",
            Style::default().fg(ACCENT).bold(),
        )));
        frame.render_widget(
            Paragraph::new(right_lines).wrap(Wrap { trim: true }),
            right_layout[0],
        );
        frame.render_widget(
            Sparkline::default()
                .data(&self.metrics.cpu_history)
                .style(Style::default().fg(ACCENT))
                .max(100),
            right_layout[1],
        );
        frame.render_widget(panel("CPU / HARDWARE", self.selected_panel == 0), area);
    }

    fn draw_memory(&self, frame: &mut Frame, area: Rect) {
        let mut lines = Vec::new();
        if let Some(m) = self.metrics.memory {
            let ram = pct(m.used_kib, m.total_kib);
            lines.push(Line::from(format!(
                "RAM       {} / {}   {ram}%",
                bytes(m.used_kib * 1024),
                bytes(m.total_kib * 1024)
            )));
            lines.push(Line::from(Span::styled(
                bar(ram as f64, area.width.saturating_sub(5) as usize),
                Style::default().fg(GREEN),
            )));
            let swap = pct(m.swap_used_kib, m.swap_total_kib);
            lines.push(Line::from(format!(
                "Swap      {} / {}   {swap}%",
                bytes(m.swap_used_kib * 1024),
                bytes(m.swap_total_kib * 1024)
            )));
            lines.push(Line::from(Span::styled(
                bar(swap as f64, area.width.saturating_sub(5) as usize),
                Style::default().fg(ACCENT),
            )));
        } else {
            lines.push(Line::from("Memory metrics unavailable"));
        }
        frame.render_widget(
            Paragraph::new(lines)
                .block(panel("MEMORY", self.selected_panel == 1))
                .wrap(Wrap { trim: true }),
            area,
        );
    }

    fn draw_disks(&self, frame: &mut Frame, area: Rect) {
        let mut lines = Vec::new();
        if let Some(disk) = &self.metrics.root_disk {
            lines.push(Line::from(format!(
                "{}  {} / {}",
                disk.device,
                bytes(disk.used_bytes),
                bytes(disk.total_bytes)
            )));
            lines.push(Line::from(vec![
                Span::styled(
                    bar(
                        disk.used_percent as f64,
                        area.width.saturating_sub(9) as usize,
                    ),
                    Style::default().fg(if disk.used_percent >= 85 { RED } else { GREEN }),
                ),
                Span::raw(format!(" {}%", disk.used_percent)),
            ]));
            lines.push(Line::from(format!("Free {}", bytes(disk.available_bytes))));
        } else {
            lines.push(Line::from("Disk metrics unavailable"));
        }
        for disk in self.metrics.disks.iter().take(2) {
            lines.push(Line::from(format!(
                "{} I/O  R {}  W {}",
                disk.name,
                disk.read_bytes_per_sec
                    .map(bytes)
                    .unwrap_or_else(|| "—".into()),
                disk.write_bytes_per_sec
                    .map(bytes)
                    .unwrap_or_else(|| "—".into())
            )));
        }
        frame.render_widget(
            Paragraph::new(lines)
                .block(panel("DISKS", self.selected_panel == 2))
                .wrap(Wrap { trim: true }),
            area,
        );
    }

    fn draw_network(&self, frame: &mut Frame, area: Rect) {
        let mut lines = Vec::new();
        for net in &self.metrics.network {
            lines.push(Line::from(Span::styled(
                format!("{}", net.name),
                Style::default().fg(ACCENT).bold(),
            )));
            lines.push(Line::from(format!(
                "↓ {:>9}/s   ↑ {:>9}/s",
                net.rx_bytes_per_sec
                    .map(bytes)
                    .unwrap_or_else(|| "—".into()),
                net.tx_bytes_per_sec
                    .map(bytes)
                    .unwrap_or_else(|| "—".into())
            )));
            lines.push(Line::from(format!(
                "Total  ↓ {}   ↑ {}",
                bytes(net.rx_bytes),
                bytes(net.tx_bytes)
            )));
        }
        if lines.is_empty() {
            lines.push(Line::from("No physical network counters available"));
        }
        frame.render_widget(
            Paragraph::new(lines)
                .block(panel("NETWORK", self.selected_panel == 3))
                .wrap(Wrap { trim: true }),
            area,
        );
    }

    fn draw_processes(&self, frame: &mut Frame, area: Rect) {
        let header = Row::new([
            Cell::from("PID"),
            Cell::from("USER"),
            Cell::from("PROCESS TREE"),
            Cell::from("MEM"),
            Cell::from("CPU"),
        ])
        .style(Style::default().fg(ACCENT).bold());
        let visible = area.height.saturating_sub(3) as usize;
        let mut rows = self
            .metrics
            .processes
            .iter()
            .skip(self.process_offset)
            .take(visible)
            .map(|p| {
                let name = format!("{}{}", "  ".repeat(p.depth.min(4)), p.name);
                Row::new(vec![
                    p.pid.to_string(),
                    p.user.clone(),
                    name,
                    format!("{:.1}%", p.memory_percent),
                    format!("{:.1}%", p.cpu_percent),
                ])
                .style(Style::default().fg(FG))
            })
            .collect::<Vec<_>>();
        if self.metrics.processes.is_empty() {
            rows.push(
                Row::new(["—", "—", "Process data unavailable", "—", "—"])
                    .style(Style::default().fg(MUTED)),
            );
        }
        let table = Table::new(
            rows,
            [
                Constraint::Length(8),
                Constraint::Length(9),
                Constraint::Min(10),
                Constraint::Length(7),
                Constraint::Length(7),
            ],
        )
        .header(header)
        .block(panel("PROCESSES · CPU SORT", self.selected_panel == 4))
        .column_spacing(1);
        frame.render_widget(table, area);
    }
}

fn panel(title: &'static str, selected: bool) -> Block<'static> {
    Block::default()
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(ACCENT).bold(),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if selected { ACCENT } else { MUTED }))
}
fn key(text: &'static str) -> Span<'static> {
    Span::styled(
        format!(" {text} "),
        Style::default().fg(BG).bg(ACCENT).bold(),
    )
}
fn hint(text: &'static str) -> Span<'static> {
    Span::styled(text, Style::default().fg(FG))
}
fn pct(used: u64, total: u64) -> u64 {
    if total == 0 {
        0
    } else {
        used.saturating_mul(100) / total
    }
}
fn bar(percent: f64, width: usize) -> String {
    let filled = (percent.clamp(0.0, 100.0) / 100.0 * width as f64).round() as usize;
    format!(
        "{}{}",
        "█".repeat(filled),
        "░".repeat(width.saturating_sub(filled))
    )
}
fn usage_color(n: f64) -> Color {
    if n >= 85.0 {
        RED
    } else if n >= 65.0 {
        ACCENT
    } else {
        GREEN
    }
}
fn bytes(bytes: u64) -> String {
    let units = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < units.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{value:.0} {}", units[unit])
    } else {
        format!("{value:.1} {}", units[unit])
    }
}
