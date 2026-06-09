use crate::commands::{CommandExecutor, CommandToExecute};
use crate::modules::TuiModule;
use crate::tui::widgets::SelectableList;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};
use ratatui::Frame;

pub struct NetworkModule {
    info_lines: Vec<(String, String)>,
    action_list: SelectableList,
    loaded: bool,
}

impl NetworkModule {
    pub fn new() -> Self {
        Self {
            info_lines: Vec::new(),
            action_list: SelectableList::new(vec![
                "Ping 8.8.8.8 (Google DNS)".to_string(),
                "Ping 223.5.5.5 (阿里 DNS)".to_string(),
                "Ping baidu.com".to_string(),
                "重启 NetworkManager".to_string(),
                "显示路由表".to_string(),
                "显示 DNS 配置".to_string(),
            ]),
            loaded: false,
        }
    }

    fn gather_info(&mut self) {
        let mut info = Vec::new();

        // IP addresses
        let ips = CommandExecutor::run_silent(
            "ip -brief addr show 2>/dev/null | grep -v lo | head -5"
        ).unwrap_or_else(|_| "未知".to_string());
        for line in ips.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                info.push((parts[0].to_string(), format!("{} ({})", parts[2..].join(", "), parts[1])));
            }
        }

        // Default gateway
        let gw = CommandExecutor::run_silent(
            "ip route show default 2>/dev/null | awk '{print $3}' | head -1"
        ).unwrap_or_else(|_| "未知".to_string());
        info.push(("默认网关".to_string(), gw));

        // DNS servers
        let dns = CommandExecutor::run_silent(
            "grep -E '^nameserver' /etc/resolv.conf 2>/dev/null | awk '{print $2}' | head -3 | paste -sd', '"
        ).unwrap_or_else(|_| "未知".to_string());
        info.push(("DNS 服务器".to_string(), dns));

        // NetworkManager status
        let nm_status = CommandExecutor::run_silent(
            "systemctl is-active NetworkManager 2>/dev/null"
        ).unwrap_or_else(|_| "未安装".to_string());
        info.push(("NetworkManager".to_string(), nm_status));

        // WiFi info
        let wifi = CommandExecutor::run_silent(
            "nmcli -t -f active,ssid dev wifi 2>/dev/null | grep '^yes' | cut -d: -f2"
        ).unwrap_or_else(|_| String::new());
        if !wifi.is_empty() {
            info.push(("WiFi SSID".to_string(), wifi));
        }

        // Hostname
        let hostname = CommandExecutor::run_silent("hostname")
            .unwrap_or_else(|_| "未知".to_string());
        info.push(("主机名".to_string(), hostname));

        self.info_lines = info;
        self.loaded = true;
    }
}

impl TuiModule for NetworkModule {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<CommandToExecute> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.action_list.previous();
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.action_list.next();
                None
            }
            KeyCode::Char('r') => {
                self.gather_info();
                None
            }
            KeyCode::Enter => {
                let idx = self.action_list.selected_index();
                let (name, cmd, warning) = match idx {
                    0 => ("Ping Google DNS", "ping -c 4 8.8.8.8", None),
                    1 => ("Ping 阿里 DNS", "ping -c 4 223.5.5.5", None),
                    2 => ("Ping baidu.com", "ping -c 4 baidu.com", None),
                    3 => ("重启 NetworkManager", "systemctl restart NetworkManager",
                          Some("重启网络管理器会导致网络短暂断开")),
                    4 => ("显示路由表", "ip route show", None),
                    5 => ("显示 DNS 配置", "cat /etc/resolv.conf", None),
                    _ => return None,
                };
                Some(CommandToExecute {
                    display_name: name.to_string(),
                    command_string: cmd.to_string(),
                    risk_warning: warning.map(|s| s.to_string()),
                })
            }
            _ => None,
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Left: Network info table
        let rows: Vec<Row> = self.info_lines.iter().map(|(k, v)| {
            Row::new(vec![
                format!("  {}", k),
                v.clone(),
            ])
        }).collect();

        let table = Table::new(
            rows,
            [Constraint::Length(18), Constraint::Min(20)],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 🌐 网络信息 ")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .border_style(Style::default().fg(Color::Blue)),
        );
        frame.render_widget(table, chunks[0]);

        // Right: Action list
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(2)])
            .split(chunks[1]);

        self.action_list.render(frame, right_chunks[0], "🔧 网络诊断");

        let hint = Paragraph::new(Line::from(Span::styled(
            "  ↑↓ 选择  │  Enter 执行  │  R 刷新信息",
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(hint, right_chunks[1]);
    }

    fn refresh(&mut self) {
        if !self.loaded {
            self.gather_info();
        }
    }
}
