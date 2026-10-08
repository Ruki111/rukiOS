use std::collections::HashSet;
use std::io;
use std::process::Command;
use std::time::{Duration, Instant};

use crate::monitor::{Metrics, Sampler};
use crate::services::{ServiceAction, ServiceUnit, list_services, perform_service_action};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Row, Sparkline, Table, Wrap},
};

const SECTIONS: [&str; 10] = [
    "Overview",
    "Monitor",
    "System",
    "Disk",
    "Memory",
    "Network",
    "Processes",
    "Services",
    "Health",
    "Logs",
];

struct App {
    selected: usize,
    scroll: u16,
    snapshot: Snapshot,
    sampler: Sampler,
    metrics: Metrics,
    last_monitor_update: Instant,
    services: Vec<ServiceUnit>,
    services_error: Option<String>,
    service_filter: String,
    service_filter_before_edit: String,
    service_filter_input: bool,
    service_selected: usize,
    service_offset: usize,
    service_visible_rows: usize,
    pending_service_action: Option<(ServiceAction, String)>,
    service_status: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        let mut sampler = Sampler::new();
        let metrics = sampler.sample();
        let (services, services_error) = match list_services() {
            Ok(services) => (services, None),
            Err(error) => (Vec::new(), Some(error)),
        };
        Self {
            selected: 1,
            scroll: 0,
            snapshot: Snapshot::load(),
            sampler,
            metrics,
            last_monitor_update: Instant::now(),
            services,
            services_error,
            service_filter: String::new(),
            service_filter_before_edit: String::new(),
            service_filter_input: false,
            service_selected: 0,
            service_offset: 0,
            service_visible_rows: 1,
            pending_service_action: None,
            service_status: None,
        }
    }
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            if self.selected == 1 && self.last_monitor_update.elapsed() >= Duration::from_secs(1) {
                self.metrics = self.sampler.sample();
                self.last_monitor_update = Instant::now();
            }
            terminal.draw(|frame| self.draw(frame))?;
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if self.pending_service_action.is_some() {
                        match key.code {
                            KeyCode::Char('y') | KeyCode::Enter => self.confirm_service_action(),
                            KeyCode::Char('n') | KeyCode::Esc => self.cancel_service_action(),
                            _ => {}
                        }
                        continue;
                    }
                    if self.selected == 7 && self.service_filter_input {
                        match key.code {
                            KeyCode::Char(character) => self.service_filter.push(character),
                            KeyCode::Backspace => {
                                self.service_filter.pop();
                            }
                            KeyCode::Enter => self.service_filter_input = false,
                            KeyCode::Esc => {
                                self.service_filter
                                    .clone_from(&self.service_filter_before_edit);
                                self.service_filter_input = false;
                            }
                            _ => {}
                        }
                        self.service_selected = 0;
                        self.service_offset = 0;
                        continue;
                    }
                    if self.selected == 7 {
                        self.service_status = None;
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            KeyCode::Char('/') => {
                                self.service_filter_before_edit
                                    .clone_from(&self.service_filter);
                                self.service_filter_input = true;
                                self.service_selected = 0;
                                self.service_offset = 0;
                            }
                            KeyCode::Char('s') => self.request_service_action(ServiceAction::Start),
                            KeyCode::Char('x') => self.request_service_action(ServiceAction::Stop),
                            KeyCode::Char('R') => {
                                self.request_service_action(ServiceAction::Restart)
                            }
                            KeyCode::Down | KeyCode::Char('j') => self.move_service_selection(1),
                            KeyCode::Up | KeyCode::Char('k') => self.move_service_selection(-1),
                            KeyCode::PageDown => {
                                self.move_service_selection(self.service_visible_rows as isize)
                            }
                            KeyCode::PageUp => {
                                self.move_service_selection(-(self.service_visible_rows as isize))
                            }
                            KeyCode::Home | KeyCode::Char('g') => {
                                self.service_selected = 0;
                                self.service_offset = 0;
                            }
                            KeyCode::End | KeyCode::Char('G') => {
                                self.service_selected =
                                    self.filtered_services().len().saturating_sub(1);
                                self.service_offset = self.service_max_offset();
                            }
                            KeyCode::Char('r') => self.reload_services(),
                            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                                self.select_next();
                            }
                            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
                                self.select_previous();
                            }
                            KeyCode::Char(number @ '1'..='9') => {
                                self.selected = number.to_digit(10).unwrap_or(1) as usize - 1;
                                self.scroll = 0;
                            }
                            KeyCode::Char('0') => {
                                self.selected = 9;
                                self.scroll = 0;
                            }
                            _ => {}
                        }
                        continue;
                    }
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Down
                        | KeyCode::Char('j')
                        | KeyCode::Char('l')
                        | KeyCode::Right => self.select_next(),
                        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('h') | KeyCode::Left => {
                            self.select_previous()
                        }
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
                            self.metrics = self.sampler.sample();
                            self.last_monitor_update = Instant::now();
                            self.scroll = 0;
                            self.reload_services();
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
                        KeyCode::Char(number @ '1'..='9') => {
                            self.selected = number.to_digit(10).unwrap_or(1) as usize - 1;
                            self.scroll = 0;
                        }
                        KeyCode::Char('0') => {
                            self.selected = 9;
                            self.scroll = 0;
                        }
                        _ => {}
                    }
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

    fn draw(&mut self, frame: &mut Frame) {
        let page = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(4),
            ])
            .split(frame.area());

        let header = Paragraph::new(Line::from(vec![
            Span::styled(
                " rukiOS ",
                Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
            ),
            Span::styled("  SYSTEM CONSOLE", Style::default().fg(Color::White).bold()),
            Span::styled(
                "                                      RUST • LINUX",
                Style::default().fg(Color::Gray),
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(header, page[0]);

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(22), Constraint::Min(0)])
            .split(page[1]);
        self.draw_navigation(frame, body[0]);
        if self.selected == 0 {
            self.draw_overview(frame, body[1]);
        } else if self.selected == 1 {
            self.draw_monitor(frame, body[1]);
        } else if self.selected == 7 {
            self.draw_services(frame, body[1]);
        } else {
            self.draw_section(frame, body[1]);
        }

        let footer_lines = if self.selected == 7 {
            if self.service_filter_input {
                vec![
                    Line::from(vec![
                        keycap(&format!("Filter: {}", self.service_filter)),
                        key_label("  Enter apply  Esc cancel"),
                    ]),
                    Line::from(vec![
                        keycap("j/k"),
                        key_label(" select  "),
                        keycap("/"),
                        key_label(" filter  "),
                        keycap("q/Esc"),
                        key_label(" quit"),
                    ]),
                ]
            } else {
                vec![
                    Line::from(vec![
                        keycap("j/k"),
                        key_label(" select  "),
                        keycap("/"),
                        key_label(" filter  "),
                        keycap("s"),
                        key_label(" start  "),
                        keycap("x"),
                        key_label(" stop  "),
                        keycap("R"),
                        key_label(" restart"),
                    ]),
                    Line::from(vec![
                        keycap("r"),
                        key_label(" refresh  "),
                        keycap("y/Enter"),
                        key_label(" confirm  "),
                        keycap("n/Esc"),
                        key_label(" cancel"),
                    ]),
                    Line::from(vec![
                        keycap("h/l"),
                        key_label(" sections  "),
                        keycap("q/Esc"),
                        key_label(" quit"),
                    ]),
                ]
            }
        } else {
            vec![
                Line::from(vec![
                    keycap("j/k"),
                    key_label(" move  "),
                    keycap("h/l"),
                    key_label(" prev/next  "),
                    keycap("←/→"),
                    key_label(" prev/next  "),
                    keycap("1-9/0"),
                    key_label(" jump to section"),
                ]),
                Line::from(vec![
                    keycap("g/G"),
                    key_label(" first/last section  "),
                    keycap("r"),
                    key_label(" refresh  "),
                    keycap("Ctrl-u/d"),
                    key_label(" scroll  "),
                    keycap("q/Esc"),
                    key_label(" quit"),
                ]),
            ]
        };
        let footer = Paragraph::new(footer_lines).block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(footer, page[2]);
        self.draw_service_confirmation(frame, frame.area());
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

    fn draw_section(&mut self, frame: &mut Frame, area: Rect) {
        let title = SECTIONS[self.selected];
        let content = self.snapshot.section_content(self.selected);
        let inner_width = area.width.saturating_sub(2).max(1);
        let inner_height = area.height.saturating_sub(2) as usize;
        let total_lines = wrapped_line_count(&content, inner_width);
        let max_scroll = total_lines.saturating_sub(inner_height);
        self.scroll = self.scroll.min(max_scroll.min(u16::MAX as usize) as u16);
        let paragraph = Paragraph::new(content)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .title(format!(" {title} "))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: false });
        let paragraph = paragraph.scroll((self.scroll, 0));
        frame.render_widget(paragraph, area);
    }

    fn draw_services(&mut self, frame: &mut Frame, area: Rect) {
        let outer = Block::default()
            .title(" SERVICES · SYSTEMD ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let inner = outer.inner(area);
        frame.render_widget(outer, area);

        let panes = if inner.width >= 90 {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
                .split(inner)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
                .split(inner)
        };
        self.service_visible_rows = panes[0].height.saturating_sub(3).max(1) as usize;
        self.clamp_service_selection();

        let visible = self.filtered_services();
        let table_rows = visible
            .iter()
            .enumerate()
            .skip(self.service_offset)
            .take(panes[0].height.saturating_sub(3) as usize)
            .map(|(row_index, service)| {
                let style = if row_index == self.service_selected {
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Rgb(35, 63, 61))
                        .add_modifier(Modifier::BOLD)
                } else if service.active == "failed" {
                    Style::default().fg(Color::Red)
                } else if service.active == "active" {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Gray)
                };
                Row::new([
                    service.name.clone(),
                    service.load.clone(),
                    service.active.clone(),
                    service.sub.clone(),
                ])
                .style(style)
            })
            .collect::<Vec<_>>();
        let service_header = Row::new(["UNIT", "LOAD", "ACTIVE", "SUB"])
            .style(Style::default().fg(Color::Cyan).bold());
        let filter_title = if self.service_filter.is_empty() {
            format!("{} services", visible.len())
        } else {
            format!("{} matches · {}", visible.len(), self.service_filter)
        };
        let service_table = Table::new(
            table_rows,
            [
                Constraint::Percentage(48),
                Constraint::Length(8),
                Constraint::Length(9),
                Constraint::Min(8),
            ],
        )
        .header(service_header)
        .column_spacing(1)
        .block(
            Block::default()
                .title(filter_title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(service_table, panes[0]);

        let detail = if let Some(error) = &self.services_error {
            Paragraph::new(error.as_str())
        } else if let Some(service) = visible.get(self.service_selected) {
            let mut lines = vec![
                Line::from(Span::styled(
                    service.name.clone(),
                    Style::default().fg(Color::Cyan).bold(),
                )),
                Line::from(format!("Load:   {}", service.load)),
                Line::from(format!("State:  {} / {}", service.active, service.sub)),
                Line::from(""),
                Line::from(Span::styled(
                    "DESCRIPTION",
                    Style::default().fg(Color::Cyan).bold(),
                )),
                Line::from(if service.description.is_empty() {
                    "No description available".to_string()
                } else {
                    service.description.clone()
                }),
            ];
            if let Some(status) = &self.service_status {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    status.clone(),
                    Style::default().fg(Color::Yellow),
                )));
            }
            Paragraph::new(lines)
        } else if self.services.is_empty() {
            Paragraph::new("No services found. Press r to retry.")
        } else {
            Paragraph::new("No services match this filter.")
        };
        let detail = detail
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .title(" SELECTED SERVICE ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        frame.render_widget(detail, panes[1]);
    }

    fn filtered_services(&self) -> Vec<&ServiceUnit> {
        let query = self.service_filter.trim().to_lowercase();
        self.services
            .iter()
            .filter(|service| {
                query.is_empty()
                    || service.name.to_lowercase().contains(&query)
                    || service.description.to_lowercase().contains(&query)
                    || service.active.to_lowercase().contains(&query)
                    || service.sub.to_lowercase().contains(&query)
            })
            .collect()
    }

    fn service_max_offset(&self) -> usize {
        self.filtered_services()
            .len()
            .saturating_sub(self.service_visible_rows.max(1))
    }

    fn clamp_service_selection(&mut self) {
        let count = self.filtered_services().len();
        if count == 0 {
            self.service_selected = 0;
            self.service_offset = 0;
            return;
        }
        self.service_selected = self.service_selected.min(count - 1);
        if self.service_selected < self.service_offset {
            self.service_offset = self.service_selected;
        } else if self.service_selected >= self.service_offset + self.service_visible_rows {
            self.service_offset = self
                .service_selected
                .saturating_add(1)
                .saturating_sub(self.service_visible_rows);
        }
        self.service_offset = self.service_offset.min(self.service_max_offset());
    }

    fn move_service_selection(&mut self, amount: isize) {
        let count = self.filtered_services().len();
        if count == 0 {
            return;
        }
        self.service_selected = (self.service_selected as isize + amount)
            .clamp(0, count.saturating_sub(1) as isize) as usize;
        self.clamp_service_selection();
    }

    fn reload_services(&mut self) {
        match list_services() {
            Ok(services) => {
                self.services = services;
                self.services_error = None;
            }
            Err(error) => {
                self.services.clear();
                self.services_error = Some(error);
            }
        }
        self.clamp_service_selection();
    }

    fn request_service_action(&mut self, action: ServiceAction) {
        let visible = self.filtered_services();
        let Some(service) = visible.get(self.service_selected) else {
            self.service_status = Some("Select a service first.".into());
            return;
        };
        self.pending_service_action = Some((action, service.name.clone()));
    }

    fn confirm_service_action(&mut self) {
        let Some((action, name)) = self.pending_service_action.take() else {
            return;
        };
        match perform_service_action(action, &name) {
            Ok(()) => {
                self.service_status = Some(format!("Requested {} for {name}.", action.as_str()));
                self.reload_services();
            }
            Err(error) => self.service_status = Some(error),
        }
    }

    fn cancel_service_action(&mut self) {
        self.pending_service_action = None;
        self.service_status = Some("Service action canceled.".into());
    }

    fn draw_service_confirmation(&self, frame: &mut Frame, area: Rect) {
        let Some((action, name)) = &self.pending_service_action else {
            return;
        };
        let width = area.width.min(68);
        let height = area.height.min(8);
        let popup = Rect::new(
            area.x + area.width.saturating_sub(width) / 2,
            area.y + area.height.saturating_sub(height) / 2,
            width,
            height,
        );
        frame.render_widget(ratatui::widgets::Clear, popup);
        let content = Paragraph::new(vec![
            Line::from(format!("Run systemctl {} {name}?", action.as_str())),
            Line::from("This can affect system services and connected apps."),
            Line::from(Span::styled(
                "y / Enter confirm    n / Esc cancel",
                Style::default().fg(Color::Yellow).bold(),
            )),
        ])
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .block(
            Block::default()
                .title(" Confirm service action ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::Black)),
        );
        frame.render_widget(content, popup);
    }

    fn draw_overview(&self, frame: &mut Frame, area: Rect) {
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(5),
            ])
            .margin(1)
            .split(area);

        let memory = &self.snapshot.memory;
        let memory_percent = memory_percent(&memory);
        let disk = &self.snapshot.disk;
        let disk_percent = disk.as_ref().map(|(_, percent)| *percent).unwrap_or(0);
        let active_interfaces = self.snapshot.active_interfaces;
        let overall = health_status(
            memory_percent,
            disk.as_ref().map(|(_, p)| *p),
            active_interfaces,
        );

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
            .block(
                Block::default()
                    .title(" MEMORY ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .gauge_style(Style::default().fg(if memory_percent >= 90 {
                Color::Red
            } else {
                Color::Blue
            }))
            .ratio(f64::from(memory_percent.min(100)) / 100.0)
            .label(format!("{memory_percent}% used"));
        frame.render_widget(memory_gauge, meters[0]);

        let disk_gauge = Gauge::default()
            .block(
                Block::default()
                    .title(" ROOT DISK ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(if disk_percent >= 85 {
                        Color::Yellow
                    } else {
                        Color::Green
                    })),
            )
            .gauge_style(Style::default().fg(if disk_percent >= 85 {
                Color::Yellow
            } else {
                Color::Green
            }))
            .ratio(f64::from(disk_percent.min(100)) / 100.0)
            .label(format!("{disk_percent}% used"));
        frame.render_widget(disk_gauge, meters[1]);

        let mut content = vec![
            Line::from(Span::styled(
                "QUICK STATUS",
                Style::default().fg(Color::Cyan).bold(),
            )),
            Line::from(format!(
                "  Network        {active_interfaces} active interfaces"
            )),
            Line::from(format!(
                "  Root disk      {}",
                disk.as_ref()
                    .map(|(line, _)| line.as_str())
                    .unwrap_or("unavailable")
            )),
            Line::from(format!("  Memory         {memory}")),
            Line::from(""),
            Line::from(Span::styled(
                "SYSTEM",
                Style::default().fg(Color::Cyan).bold(),
            )),
        ];
        content.extend(
            self.snapshot
                .system
                .lines()
                .map(|line| Line::from(format!("  {line}"))),
        );
        content.push(Line::from(""));
        content.push(Line::from(Span::styled(
            "Choose a section on the left to inspect details.",
            Style::default().fg(Color::Gray),
        )));
        let details = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(details, sections[2]);
    }

    fn draw_monitor(&self, frame: &mut Frame, area: Rect) {
        let panels = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .margin(1)
            .split(area);
        let lower = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
            .split(panels[1]);

        let cpu = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(3)])
            .split(panels[0]);
        let cpu_percent = self
            .metrics
            .cpu_percent
            .map(|value| format!("{value:.0}%"))
            .unwrap_or_else(|| "N/A".into());
        let cpu_label = format!(
            "CPU  {cpu_percent}   Load {}   Uptime {}",
            self.metrics.load_average, self.metrics.uptime
        );
        let cpu_gauge = Gauge::default()
            .block(
                Block::default()
                    .title(cpu_label)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .gauge_style(Style::default().fg(Color::Cyan))
            .ratio(self.metrics.cpu_percent.unwrap_or(0.0).clamp(0.0, 100.0) / 100.0)
            .label(cpu_percent);
        frame.render_widget(cpu_gauge, cpu[0]);

        let cpu_columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(57), Constraint::Percentage(43)])
            .split(cpu[1]);
        let core_rows = self
            .metrics
            .cores
            .iter()
            .map(|(name, usage)| {
                let label = usage
                    .map(|v| format!("{v:>3.0}%"))
                    .unwrap_or_else(|| " N/A".into());
                let bar = usage.map(|v| core_bar(v, 14)).unwrap_or_default();
                Line::from(vec![
                    Span::styled(format!("{name:<5}"), Style::default().fg(Color::Gray)),
                    Span::styled(bar, Style::default().fg(usage_color(usage.unwrap_or(0.0)))),
                    Span::raw(format!(" {label}")),
                ])
            })
            .collect::<Vec<_>>();
        let cores = Paragraph::new(core_rows)
            .block(
                Block::default()
                    .title(" PER CORE ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .scroll((0, 0));
        frame.render_widget(cores, cpu_columns[0]);

        let mut hardware = vec![Line::from(Span::styled(
            "GPU",
            Style::default().fg(Color::Cyan).bold(),
        ))];
        if self.metrics.gpus.is_empty() {
            hardware.push(Line::from("  GPU metrics unavailable"));
        } else {
            for gpu in &self.metrics.gpus {
                let util = gpu
                    .utilization_percent
                    .map(|n| format!("{n}%"))
                    .unwrap_or_else(|| "N/A".into());
                let memory = match (gpu.memory_used_bytes, gpu.memory_total_bytes) {
                    (Some(used), Some(total)) => {
                        format!("{} / {}", format_bytes(used), format_bytes(total))
                    }
                    _ => "N/A".into(),
                };
                let temp = gpu
                    .temperature_celsius
                    .map(|n| format!("{n:.0}°C"))
                    .unwrap_or_else(|| "N/A".into());
                hardware.push(Line::from(format!(
                    "  {}  {util}  VRAM {memory}  {temp}",
                    gpu.name
                )));
            }
        }
        hardware.push(Line::from(""));
        hardware.push(Line::from(Span::styled(
            "TEMPERATURES",
            Style::default().fg(Color::Cyan).bold(),
        )));
        if self.metrics.temperatures.is_empty() {
            hardware.push(Line::from("  Sensors unavailable"));
        } else {
            for sensor in self.metrics.temperatures.iter().take(5) {
                hardware.push(Line::from(format!(
                    "  {}  {:.0}°C",
                    sensor.label, sensor.celsius
                )));
            }
        }
        let hw_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(cpu_columns[1]);
        let hw = Paragraph::new(hardware)
            .block(
                Block::default()
                    .title(" GPU / SENSORS ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(hw, hw_area[0]);
        let history = Sparkline::default()
            .block(
                Block::default()
                    .title(" CPU HISTORY · 60s ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .data(&self.metrics.cpu_history)
            .style(Style::default().fg(Color::Cyan))
            .max(100);
        frame.render_widget(history, hw_area[1]);

        let left_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(lower[0]);
        self.draw_memory_panel(frame, left_rows[0]);
        self.draw_io_panel(frame, left_rows[1]);
        self.draw_process_panel(frame, lower[1]);
    }

    fn draw_memory_panel(&self, frame: &mut Frame, area: Rect) {
        let Some(memory) = self.metrics.memory else {
            frame.render_widget(
                Paragraph::new("Memory metrics unavailable")
                    .block(Block::default().title(" MEMORY ").borders(Borders::ALL)),
                area,
            );
            return;
        };
        let used = (memory.used_kib / 1024, memory.total_kib / 1024);
        let memory_pct = if memory.total_kib == 0 {
            0
        } else {
            (memory.used_kib.saturating_mul(100) / memory.total_kib).min(100) as u16
        };
        let swap_pct = if memory.swap_total_kib == 0 {
            0
        } else {
            (memory.swap_used_kib.saturating_mul(100) / memory.swap_total_kib).min(100) as u16
        };
        let mut lines = vec![Line::from(format!(
            "RAM   {} / {} MiB   {memory_pct}%",
            used.0, used.1
        ))];
        let ram_gauge = Gauge::default()
            .ratio(f64::from(memory_pct) / 100.0)
            .gauge_style(Style::default().fg(if memory_pct >= 90 {
                Color::Red
            } else {
                Color::Green
            }))
            .label(format!("{memory_pct}%"));
        let inner = Block::default()
            .title(" MEMORY ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .margin(1)
            .split(area);
        frame.render_widget(inner, area);
        frame.render_widget(Paragraph::new(lines.remove(0)), chunks[0]);
        frame.render_widget(ram_gauge, chunks[1]);
        if memory.swap_total_kib > 0 {
            frame.render_widget(
                Paragraph::new(format!(
                    "Swap  {} / {} MiB   {swap_pct}%",
                    memory.swap_used_kib / 1024,
                    memory.swap_total_kib / 1024
                )),
                chunks[2],
            );
        } else {
            frame.render_widget(Paragraph::new("Swap  not configured"), chunks[2]);
        }
    }

    fn draw_io_panel(&self, frame: &mut Frame, area: Rect) {
        let mut lines = vec![Line::from(Span::styled(
            "ROOT FILESYSTEM",
            Style::default().fg(Color::Cyan).bold(),
        ))];
        if let Some(disk) = &self.metrics.root_disk {
            lines.push(Line::from(format!(
                "{}  {} / {}  {} free  {}% used",
                disk.device,
                format_bytes(disk.used_bytes),
                format_bytes(disk.total_bytes),
                format_bytes(disk.available_bytes),
                disk.used_percent
            )));
        } else {
            lines.push(Line::from("Root filesystem metrics unavailable"));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "DISK I/O",
            Style::default().fg(Color::Cyan).bold(),
        )));
        for disk in self.metrics.disks.iter().take(4) {
            lines.push(Line::from(format!(
                "{}  R {}/s  W {} /s",
                disk.name,
                disk.read_bytes_per_sec
                    .map(format_bytes)
                    .unwrap_or_else(|| "...".into()),
                disk.write_bytes_per_sec
                    .map(format_bytes)
                    .unwrap_or_else(|| "...".into())
            )));
        }
        if !self.metrics.network.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "NETWORK  (RX / TX)",
                Style::default().fg(Color::Cyan).bold(),
            )));
            for net in self.metrics.network.iter().take(2) {
                lines.push(Line::from(format!(
                    "{}  {}/s  {}/s",
                    net.name,
                    net.rx_bytes_per_sec
                        .map(format_bytes)
                        .unwrap_or_else(|| "...".into()),
                    net.tx_bytes_per_sec
                        .map(format_bytes)
                        .unwrap_or_else(|| "...".into())
                )));
            }
        }
        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" DISKS / NETWORK ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
    }

    fn draw_process_panel(&self, frame: &mut Frame, area: Rect) {
        let header = Row::new(["PID", "USER", "PROCESS", "CPU%", "MEM%"])
            .style(Style::default().fg(Color::Cyan).bold());
        let rows = self.metrics.processes.iter().map(|proc| {
            let name = format!("{}{}", "  ".repeat(proc.depth.min(3)), proc.name);
            Row::new(vec![
                proc.pid.to_string(),
                proc.user.clone(),
                name,
                format!("{:.1}", proc.cpu_percent),
                format!("{:.1}", proc.memory_percent),
            ])
        });
        let table = Table::new(
            rows,
            [
                Constraint::Length(7),
                Constraint::Length(8),
                Constraint::Min(8),
                Constraint::Length(6),
                Constraint::Length(6),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .title(" PROCESSES · CPU SORT ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .column_spacing(1);
        frame.render_widget(table, area);
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
    health: String,
    logs: String,
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
        let health = health_summary(&memory, disk.as_ref(), active_interfaces);
        let logs = logs_listing(40);

        Self {
            system,
            disk_listing,
            disk,
            memory,
            network_listing,
            active_interfaces,
            processes,
            health,
            logs,
        }
    }

    fn section_content(&self, section: usize) -> String {
        match section {
            2 => self.system.clone(),
            3 => self.disk_listing.clone(),
            4 => format!("Memory usage\n\n{}", self.memory),
            5 => self.network_listing.clone(),
            6 => self.processes.clone(),
            8 => self.health.clone(),
            9 => self.logs.clone(),
            _ => String::new(),
        }
    }
}

fn wrapped_line_count(content: &str, width: u16) -> usize {
    let width = usize::from(width.max(1));
    content
        .split('\n')
        .map(|line| Line::from(line).width().max(1).div_ceil(width))
        .sum()
}

pub fn run() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}

fn keycap(key: &str) -> Span<'static> {
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

fn core_bar(percent: f64, width: usize) -> String {
    let filled = (percent.clamp(0.0, 100.0) / 100.0 * width as f64).round() as usize;
    format!(
        "{}{}",
        "█".repeat(filled),
        "░".repeat(width.saturating_sub(filled))
    )
}

fn usage_color(percent: f64) -> Color {
    if percent >= 85.0 {
        Color::Red
    } else if percent >= 65.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn system_summary(memory: &str) -> String {
    let os = std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|text| {
            text.lines()
                .find_map(|line| {
                    line.strip_prefix("PRETTY_NAME=\"")
                        .and_then(|s| s.strip_suffix('"'))
                })
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "Unknown Linux distribution".into());
    let kernel = run_command("uname", &["-r"]).unwrap_or_else(|| "unknown".into());
    let uptime = std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse::<f64>().ok())
        .map(|seconds| {
            format!(
                "{} hours, {} minutes",
                seconds as u64 / 3600,
                (seconds as u64 % 3600) / 60
            )
        })
        .unwrap_or_else(|| "unknown".into());
    format!(
        "Operating system  {os}\nKernel            {kernel}\nUptime            {uptime}\nMemory            {memory}"
    )
}

fn disk_listing() -> String {
    let Some(output) = run_command(
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
    ) else {
        return "Could not read disk information (is `df` installed?)".into();
    };
    let mut seen = HashSet::new();
    output
        .lines()
        .filter(|line| {
            line.split_whitespace()
                .next()
                .is_some_and(|source| seen.insert(source.to_string()))
        })
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
            total = value
                .split_whitespace()
                .next()
                .and_then(|n| n.parse::<u64>().ok());
        } else if let Some(value) = line.strip_prefix("MemAvailable:") {
            available = value
                .split_whitespace()
                .next()
                .and_then(|n| n.parse::<u64>().ok());
        }
    }
    match (total, available) {
        (Some(total), Some(available)) => format!(
            "{} MiB used / {} MiB total",
            total.saturating_sub(available) / 1024,
            total / 1024
        ),
        _ => "Memory information unavailable".into(),
    }
}

fn memory_percent(memory: &str) -> u16 {
    let values = memory.split_whitespace().collect::<Vec<_>>();
    let used = values.first().and_then(|n| n.parse::<u64>().ok());
    let total = values.get(4).and_then(|n| n.parse::<u64>().ok());
    match (used, total) {
        (Some(used), Some(total)) if total > 0 => {
            ((used.saturating_mul(100) / total).min(100)) as u16
        }
        _ => 0,
    }
}

fn root_disk_reading() -> Option<(String, u16)> {
    let output = run_command(
        "df",
        &["-h", "--output=source,size,used,avail,pcent,target", "/"],
    )?;
    let line = output.lines().nth(1)?.trim().to_string();
    let percent = line
        .split_whitespace()
        .nth(4)?
        .trim_end_matches('%')
        .parse::<u16>()
        .ok()?;
    Some((line, percent))
}

fn network_listing() -> String {
    run_command("ip", &["-brief", "address", "show"])
        .map(|output| {
            output
                .lines()
                .filter(|line| line.split_whitespace().nth(1) == Some("UP"))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .filter(|output| !output.is_empty())
        .unwrap_or_else(|| "No active network interfaces found.".into())
}

fn active_network_count() -> usize {
    run_command("ip", &["-brief", "address", "show"])
        .map(|output| {
            output
                .lines()
                .filter(|line| line.split_whitespace().nth(1) == Some("UP"))
                .count()
        })
        .unwrap_or(0)
}

fn process_listing() -> String {
    run_command(
        "ps",
        &["-eo", "pid,comm,%cpu,%mem", "--sort=-%cpu", "--no-headers"],
    )
    .map(|output| {
        let rows = output.lines().take(20).collect::<Vec<_>>().join("\n");
        format!("PID COMMAND         %CPU %MEM\n{rows}")
    })
    .unwrap_or_else(|| "Could not list processes (is `ps` installed?)".into())
}

fn logs_listing(limit: usize) -> String {
    let limit_arg = limit.to_string();
    run_command(
        "journalctl",
        &["--no-pager", "-n", &limit_arg, "-o", "short-iso"],
    )
    .filter(|output| !output.is_empty())
    .unwrap_or_else(|| {
        "Could not read system logs. Check journalctl availability and permissions.".into()
    })
}

fn health_summary(memory: &str, disk: Option<&(String, u16)>, interfaces: usize) -> String {
    let memory_percent = memory_percent(&memory);
    let disk_percent = disk.map(|(_, percent)| *percent);
    let overall = health_status(memory_percent, disk_percent, interfaces);
    format!(
        "Overall     {overall}\nMemory      {}  {memory}\nDisk        {}  {}\nNetwork     {} active interface(s)",
        if memory_percent >= 90 { "WARN" } else { "OK" },
        disk_percent
            .map(|p| if p >= 85 { "WARN" } else { "OK" })
            .unwrap_or("UNKNOWN"),
        disk.map(|(line, _)| line.as_str()).unwrap_or("unavailable"),
        if interfaces == 0 { "WARN" } else { "OK" },
    )
}

fn health_status(
    memory_percent: u16,
    disk_percent: Option<u16>,
    interfaces: usize,
) -> &'static str {
    if memory_percent >= 90 || disk_percent.is_some_and(|percent| percent >= 85) || interfaces == 0
    {
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
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}
