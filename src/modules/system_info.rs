use crate::commands::{CommandExecutor, CommandToExecute};
use crate::modules::TuiModule;
use crossterm::event::KeyEvent;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};
use ratatui::Frame;
use sysinfo::System;
use std::fs;

pub struct SystemInfoModule {
    info_lines: Vec<(String, String)>,
    loaded: bool,
}

impl SystemInfoModule {
    pub fn new() -> Self {
        Self {
            info_lines: Vec::new(),
            loaded: false,
        }
    }

    fn gather_info(&mut self) {
        let mut info = Vec::new();

        // Distribution
        let distro = Self::read_os_field("PRETTY_NAME")
            .unwrap_or_else(|| "Unknown".to_string());
        info.push(("发行版".to_string(), distro));

        // Kernel
        let kernel = CommandExecutor::run_silent("uname -r")
            .unwrap_or_else(|_| "Unknown".to_string());
        info.push(("内核版本".to_string(), kernel));

        // Architecture
        let arch = CommandExecutor::run_silent("uname -m")
            .unwrap_or_else(|_| "Unknown".to_string());
        info.push(("架构".to_string(), arch));

        // Hostname
        let hostname = CommandExecutor::run_silent("hostname")
            .unwrap_or_else(|_| "Unknown".to_string());
        info.push(("主机名".to_string(), hostname));

        // Current user
        let user = CommandExecutor::run_silent("whoami")
            .unwrap_or_else(|_| "Unknown".to_string());
        info.push(("当前用户".to_string(), user));

        // Desktop environment
        let de = std::env::var("XDG_CURRENT_DESKTOP")
            .or_else(|_| std::env::var("DESKTOP_SESSION"))
            .unwrap_or_else(|_| "Unknown".to_string());
        info.push(("桌面环境".to_string(), de));

        // Session type (Wayland / X11)
        let session = std::env::var("XDG_SESSION_TYPE")
            .unwrap_or_else(|_| "Unknown".to_string());
        info.push(("会话类型".to_string(), session));

        // CPU
        let mut sys = System::new();
        sys.refresh_all();
        let cpu_name = if !sys.cpus().is_empty() {
            format!("{} ({} cores)", sys.cpus()[0].brand(), sys.cpus().len())
        } else {
            "Unknown".to_string()
        };
        info.push(("CPU".to_string(), cpu_name));

        // Memory
        sys.refresh_memory();
        let total_mem = sys.total_memory() / 1024 / 1024;
        let used_mem = sys.used_memory() / 1024 / 1024;
        info.push((
            "内存".to_string(),
            format!("{} MB / {} MB ({:.0}%)", used_mem, total_mem,
                    (used_mem as f64 / total_mem as f64) * 100.0),
        ));

        // Swap
        let total_swap = sys.total_swap() / 1024 / 1024;
        let used_swap = sys.used_swap() / 1024 / 1024;
        info.push((
            "交换空间".to_string(),
            if total_swap > 0 {
                format!("{} MB / {} MB", used_swap, total_swap)
            } else {
                "未启用".to_string()
            },
        ));

        // Disk usage (root /)
        let disk_info = CommandExecutor::run_silent("df -h / | awk 'NR==2{print $3\" / \"$2\" (\"$5\" used)\"}'")
            .unwrap_or_else(|_| "Unknown".to_string());
        info.push(("磁盘 (/)".to_string(), disk_info));

        // GPU
        let gpu = CommandExecutor::run_silent("lspci 2>/dev/null | grep -i 'vga\\|3d\\|display' | head -1 | sed 's/.*: //'")
            .unwrap_or_else(|_| "Unknown".to_string());
        info.push(("显卡".to_string(), if gpu.is_empty() { "Unknown".to_string() } else { gpu }));

        // Uptime
        let uptime = CommandExecutor::run_silent("uptime -p 2>/dev/null || uptime")
            .unwrap_or_else(|_| "Unknown".to_string());
        info.push(("运行时间".to_string(), uptime));

        // systemd status
        let systemd_status = CommandExecutor::run_silent("systemctl is-system-running 2>/dev/null")
            .unwrap_or_else(|_| "Unknown".to_string());
        info.push(("systemd 状态".to_string(), systemd_status));

        self.info_lines = info;
        self.loaded = true;
    }

    fn read_os_field(field: &str) -> Option<String> {
        let content = fs::read_to_string("/etc/os-release").ok()?;
        for line in content.lines() {
            if line.starts_with(field) {
                let val = line.splitn(2, '=').nth(1)?;
                return Some(val.trim_matches('"').to_string());
            }
        }
        None
    }
}

impl TuiModule for SystemInfoModule {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<CommandToExecute> {
        match key.code {
            crossterm::event::KeyCode::Char('r') => {
                self.gather_info();
                None
            }
            _ => None,
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(2)])
            .split(area);

        let rows: Vec<Row> = self
            .info_lines
            .iter()
            .map(|(k, v)| {
                Row::new(vec![
                    Span::styled(
                        format!("  {}", k),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    )
                    .to_string(),
                    v.clone(),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Length(16), Constraint::Min(20)],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 📊 系统信息 ")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .border_style(Style::default().fg(Color::Blue)),
        );

        frame.render_widget(table, chunks[0]);

        let hint = Paragraph::new(Line::from(Span::styled(
            "  按 [R] 刷新系统信息",
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(hint, chunks[1]);
    }

    fn refresh(&mut self) {
        if !self.loaded {
            self.gather_info();
        }
    }
}
