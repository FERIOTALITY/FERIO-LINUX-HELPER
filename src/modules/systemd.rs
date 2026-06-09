use crate::commands::{CommandExecutor, CommandToExecute};
use crate::modules::TuiModule;
use crate::tui::widgets::SelectableList;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

enum SystemdView {
    /// Main list of services
    ServiceList,
    /// Action menu for selected service
    ActionMenu,
}

struct ServiceInfo {
    name: String,
    status: String,   // "active", "inactive", "failed", etc.
    enabled: String,  // "enabled", "disabled", etc.
}

pub struct SystemdModule {
    pub selected_index: usize,
    services: Vec<ServiceInfo>,
    list: SelectableList,
    action_list: SelectableList,
    view: SystemdView,
    search_query: String,
    is_searching: bool,
}

impl SystemdModule {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            services: Vec::new(),
            list: SelectableList::new(Vec::new()),
            action_list: SelectableList::new(vec![
                "查看状态 (status)".to_string(),
                "启动 (start)".to_string(),
                "停止 (stop)".to_string(),
                "重启 (restart)".to_string(),
                "启用开机自启 (enable)".to_string(),
                "禁用开机自启 (disable)".to_string(),
                "返回服务列表".to_string(),
            ]),
            view: SystemdView::ServiceList,
            search_query: String::new(),
            is_searching: false,
        }
    }

    fn load_services(&mut self) {
        let output = CommandExecutor::run_silent(
            "systemctl list-units --type=service --all --no-pager --no-legend 2>/dev/null | head -100"
        );

        self.services.clear();
        if let Ok(text) = output {
            for line in text.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let name = parts[0].trim_start_matches('●').trim().to_string();
                    let active = parts[2].to_string();
                    // Check enabled state
                    let enabled = CommandExecutor::run_silent(
                        &format!("systemctl is-enabled {} 2>/dev/null", name)
                    ).unwrap_or_else(|_| "unknown".to_string());

                    self.services.push(ServiceInfo {
                        name,
                        status: active,
                        enabled,
                    });
                }
            }
        }

        self.update_list_items();
    }

    fn update_list_items(&mut self) {
        let filtered: Vec<String> = self.services.iter()
            .filter(|s| {
                if self.search_query.is_empty() {
                    true
                } else {
                    s.name.to_lowercase().contains(&self.search_query.to_lowercase())
                }
            })
            .map(|s| {
                let status_icon = match s.status.as_str() {
                    "active" => "🟢",
                    "inactive" => "⚪",
                    "failed" => "🔴",
                    _ => "⚫",
                };
                format!("{} {} [{}] [{}]", status_icon, s.name, s.status, s.enabled)
            })
            .collect();

        self.list = SelectableList::new(filtered);
    }

    fn get_selected_service_name(&self) -> Option<String> {
        let idx = self.list.selected_index();
        let filtered: Vec<&ServiceInfo> = self.services.iter()
            .filter(|s| {
                if self.search_query.is_empty() {
                    true
                } else {
                    s.name.to_lowercase().contains(&self.search_query.to_lowercase())
                }
            })
            .collect();
        filtered.get(idx).map(|s| s.name.clone())
    }
}

impl TuiModule for SystemdModule {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<CommandToExecute> {
        // Search mode
        if self.is_searching {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.is_searching = false;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.update_list_items();
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.update_list_items();
                }
                _ => {}
            }
            return None;
        }

        match self.view {
            SystemdView::ServiceList => {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.list.previous();
                        self.selected_index = self.list.selected_index();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.list.next();
                        self.selected_index = self.list.selected_index();
                    }
                    KeyCode::Char('/') => {
                        self.is_searching = true;
                    }
                    KeyCode::Enter => {
                        if self.get_selected_service_name().is_some() {
                            self.view = SystemdView::ActionMenu;
                            self.action_list.state.select(Some(0));
                        }
                    }
                    _ => {}
                }
                None
            }
            SystemdView::ActionMenu => {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.action_list.previous();
                        None
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.action_list.next();
                        None
                    }
                    KeyCode::Esc => {
                        self.view = SystemdView::ServiceList;
                        None
                    }
                    KeyCode::Enter => {
                        let action_idx = self.action_list.selected_index();
                        if action_idx == 6 {
                            // "返回"
                            self.view = SystemdView::ServiceList;
                            return None;
                        }
                        if let Some(svc_name) = self.get_selected_service_name() {
                            let (action_name, cmd, warning) = match action_idx {
                                0 => ("查看状态", format!("systemctl status {}", svc_name), None),
                                1 => ("启动服务", format!("systemctl start {}", svc_name),
                                      Some("启动服务可能影响系统运行".to_string())),
                                2 => ("停止服务", format!("systemctl stop {}", svc_name),
                                      Some("停止服务可能导致功能不可用".to_string())),
                                3 => ("重启服务", format!("systemctl restart {}", svc_name),
                                      Some("重启服务会导致短暂中断".to_string())),
                                4 => ("启用开机自启", format!("systemctl enable {}", svc_name), None),
                                5 => ("禁用开机自启", format!("systemctl disable {}", svc_name),
                                      Some("禁用后重启系统服务将不会自动启动".to_string())),
                                _ => return None,
                            };
                            Some(CommandToExecute {
                                display_name: format!("{}: {}", action_name, svc_name),
                                command_string: cmd,
                                risk_warning: warning,
                            })
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Search bar
                Constraint::Min(5),    // List/Actions
                Constraint::Length(2), // Hint
            ])
            .split(area);

        // Search bar
        let search_style = if self.is_searching {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let search_text = if self.search_query.is_empty() {
            if self.is_searching {
                "  搜索: _".to_string()
            } else {
                format!("  按 / 搜索  │  共 {} 个服务", self.services.len())
            }
        } else {
            format!("  搜索: {}{}", self.search_query, if self.is_searching { "_" } else { "" })
        };
        let search_bar = Paragraph::new(Line::from(Span::styled(search_text, search_style)))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            );
        frame.render_widget(search_bar, chunks[0]);

        // Main content
        match self.view {
            SystemdView::ServiceList => {
                self.list.render(frame, chunks[1], "⚙ systemd 服务管理");
            }
            SystemdView::ActionMenu => {
                let svc = self.get_selected_service_name().unwrap_or_default();
                let title = format!("⚙ {} - 操作", svc);
                self.action_list.render(frame, chunks[1], &title);
            }
        }

        // Hint
        let hint_text = match self.view {
            SystemdView::ServiceList => "  ↑↓ 选择  │  Enter 进入操作菜单  │  / 搜索",
            SystemdView::ActionMenu => "  ↑↓ 选择  │  Enter 执行  │  Esc 返回",
        };
        let hint = Paragraph::new(Line::from(Span::styled(
            hint_text,
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(hint, chunks[2]);
    }

    fn refresh(&mut self) {
        self.load_services();
    }
}
