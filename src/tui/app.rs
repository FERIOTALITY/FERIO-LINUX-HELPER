use crate::commands::{CommandExecutor, CommandToExecute};
use crate::modules::TuiModule;
use crate::modules::desktop_fixes::DesktopFixesModule;
use crate::modules::log_viewer::LogViewerModule;
use crate::modules::network::NetworkModule;
use crate::modules::optimizations::OptimizationsModule;
use crate::modules::package_manager::PackageManagerModule;
use crate::modules::system_info::SystemInfoModule;
use crate::modules::systemd::SystemdModule;
use crate::tui::widgets::centered_rect;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Tabs, Wrap};
use ratatui::Frame;

const TAB_TITLES: [&str; 7] = [
    "1·系统信息",
    "2·软件包",
    "3·systemd",
    "4·网络",
    "5·桌面修复",
    "6·日志",
    "7·优化",
];

pub struct App {
    pub active_tab: usize,
    pub sys_info: SystemInfoModule,
    pub pkg_mgr: PackageManagerModule,
    pub systemd: SystemdModule,
    pub network: NetworkModule,
    pub desktop_fixes: DesktopFixesModule,
    pub log_viewer: LogViewerModule,
    pub optimizations: OptimizationsModule,
    pub executor: CommandExecutor,
    pub modal_active: bool,
    pub pending_command: Option<CommandToExecute>,
    pub status_message: String,
    pub should_quit: bool,
    dry_run: bool,
}

impl App {
    pub fn new(dry_run: bool) -> Self {
        Self {
            active_tab: 0,
            sys_info: SystemInfoModule::new(),
            pkg_mgr: PackageManagerModule::new(),
            systemd: SystemdModule::new(),
            network: NetworkModule::new(),
            desktop_fixes: DesktopFixesModule::new(),
            log_viewer: LogViewerModule::new(),
            optimizations: OptimizationsModule::new(),
            executor: CommandExecutor::new(dry_run),
            modal_active: false,
            pending_command: None,
            status_message: if dry_run {
                "模式: DRY-RUN (命令不会实际执行)".to_string()
            } else {
                "就绪".to_string()
            },
            should_quit: false,
            dry_run,
        }
    }

    /// Refresh data for the currently active module
    pub fn refresh_active_module(&mut self) {
        match self.active_tab {
            0 => self.sys_info.refresh(),
            1 => self.pkg_mgr.refresh(),
            2 => self.systemd.refresh(),
            3 => self.network.refresh(),
            4 => self.desktop_fixes.refresh(),
            5 => self.log_viewer.refresh(),
            6 => self.optimizations.refresh(),
            _ => {}
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        // Modal intercept
        if self.modal_active {
            match key.code {
                KeyCode::Char('y') | KeyCode::Enter => {
                    if let Some(cmd) = self.pending_command.take() {
                        match self.executor.execute(&cmd) {
                            Ok(msg) => {
                                self.status_message = if msg.len() > 80 {
                                    format!("{}...", &msg[..77])
                                } else {
                                    msg
                                };
                            }
                            Err(err) => {
                                self.status_message = format!("✗ {}", if err.len() > 75 {
                                    format!("{}...", &err[..72])
                                } else {
                                    err
                                });
                            }
                        }
                        // Refresh module data after command execution
                        self.refresh_active_module();
                    }
                    self.modal_active = false;
                }
                KeyCode::Char('n') | KeyCode::Esc => {
                    self.pending_command = None;
                    self.status_message = "操作已取消".to_string();
                    self.modal_active = false;
                }
                _ => {}
            }
            return;
        }

        // Global keys
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
                return;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return;
            }
            KeyCode::Tab | KeyCode::Right => {
                self.active_tab = (self.active_tab + 1) % 7;
                self.refresh_active_module();
                return;
            }
            KeyCode::BackTab | KeyCode::Left => {
                self.active_tab = if self.active_tab == 0 { 6 } else { self.active_tab - 1 };
                self.refresh_active_module();
                return;
            }
            KeyCode::Char(c) if ('1'..='7').contains(&c) => {
                self.active_tab = (c as usize) - ('1' as usize);
                self.refresh_active_module();
                return;
            }
            _ => {}
        }

        // Route to active module
        let cmd = match self.active_tab {
            0 => self.sys_info.handle_key_event(key),
            1 => self.pkg_mgr.handle_key_event(key),
            2 => self.systemd.handle_key_event(key),
            3 => self.network.handle_key_event(key),
            4 => self.desktop_fixes.handle_key_event(key),
            5 => self.log_viewer.handle_key_event(key),
            6 => self.optimizations.handle_key_event(key),
            _ => None,
        };

        if let Some(c) = cmd {
            self.pending_command = Some(c);
            self.modal_active = true;
        }
    }

    pub fn tick(&mut self) {
        // Periodic background updates can be added here
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let size = frame.size();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Header tabs
                Constraint::Min(10),   // Body
                Constraint::Length(3), // Status bar
            ])
            .split(size);

        // Tab bar
        let titles: Vec<Line> = TAB_TITLES
            .iter()
            .enumerate()
            .map(|(i, t)| {
                if i == self.active_tab {
                    Line::from(Span::styled(
                        *t,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else {
                    Line::from(Span::styled(*t, Style::default().fg(Color::DarkGray)))
                }
            })
            .collect();

        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" ferio-linux-helper ")
                    .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .select(self.active_tab)
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .divider(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
        frame.render_widget(tabs, chunks[0]);

        // Body
        match self.active_tab {
            0 => self.sys_info.draw(frame, chunks[1]),
            1 => self.pkg_mgr.draw(frame, chunks[1]),
            2 => self.systemd.draw(frame, chunks[1]),
            3 => self.network.draw(frame, chunks[1]),
            4 => self.desktop_fixes.draw(frame, chunks[1]),
            5 => self.log_viewer.draw(frame, chunks[1]),
            6 => self.optimizations.draw(frame, chunks[1]),
            _ => {}
        }

        // Status bar
        let mode_indicator = if self.dry_run { " [DRY-RUN] " } else { "" };
        let status_text = format!(
            " {}{}  │  ←→/Tab 切换  │  ↑↓ 选择  │  Enter 确认  │  Esc/q 退出",
            mode_indicator, self.status_message
        );
        let status = Paragraph::new(status_text)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        frame.render_widget(status, chunks[2]);

        // Confirmation modal overlay
        if self.modal_active {
            if let Some(ref cmd) = self.pending_command {
                let modal_area = centered_rect(65, 40, size);
                frame.render_widget(Clear, modal_area);

                let modal_block = Block::default()
                    .title(" ⚡ 确认执行 ")
                    .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow));

                let mut lines = vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("  操作: {}", cmd.display_name),
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  命令: ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            cmd.command_string.clone(),
                            Style::default().fg(Color::Green),
                        ),
                    ]),
                    Line::from(""),
                ];

                if let Some(ref warn) = cmd.risk_warning {
                    lines.push(Line::from(Span::styled(
                        format!("  ⚠ 风险提示: {}", warn),
                        Style::default().fg(Color::Red),
                    )));
                    lines.push(Line::from(""));
                }

                if self.dry_run {
                    lines.push(Line::from(Span::styled(
                        "  📋 DRY-RUN 模式: 命令不会实际执行，仅记录日志",
                        Style::default().fg(Color::Blue),
                    )));
                    lines.push(Line::from(""));
                }

                lines.push(Line::from(Span::styled(
                    "  按 [Y/Enter] 确认  │  按 [N/Esc] 取消",
                    Style::default().fg(Color::DarkGray),
                )));

                let modal_paragraph = Paragraph::new(lines)
                    .block(modal_block)
                    .wrap(Wrap { trim: false });
                frame.render_widget(modal_paragraph, modal_area);
            }
        }
    }
}
