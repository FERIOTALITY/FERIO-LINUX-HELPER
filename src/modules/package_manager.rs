use crate::commands::{CommandExecutor, CommandToExecute};
use crate::modules::TuiModule;
use crate::tui::widgets::SelectableList;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

#[derive(Debug, Clone, Copy, PartialEq)]
enum PackageManager {
    Pacman,
    Dnf,
    Apt,
    Zypper,
    Unknown,
}

impl PackageManager {
    fn detect() -> Self {
        if CommandExecutor::run_silent("command -v pacman").is_ok() {
            PackageManager::Pacman
        } else if CommandExecutor::run_silent("command -v dnf").is_ok() {
            PackageManager::Dnf
        } else if CommandExecutor::run_silent("command -v apt").is_ok() {
            PackageManager::Apt
        } else if CommandExecutor::run_silent("command -v zypper").is_ok() {
            PackageManager::Zypper
        } else {
            PackageManager::Unknown
        }
    }

    fn name(&self) -> &str {
        match self {
            PackageManager::Pacman => "pacman",
            PackageManager::Dnf => "dnf",
            PackageManager::Apt => "apt",
            PackageManager::Zypper => "zypper",
            PackageManager::Unknown => "unknown",
        }
    }

    fn actions(&self) -> Vec<(&str, &str, Option<&str>)> {
        // (display_name, command, risk_warning)
        match self {
            PackageManager::Pacman => vec![
                ("刷新软件源", "pacman -Sy", None),
                ("检查可更新包", "pacman -Qu", None),
                ("升级系统", "pacman -Syu --noconfirm", Some("将升级所有软件包，可能影响系统稳定性")),
                ("清理包缓存", "pacman -Sc --noconfirm", Some("将删除旧版本包缓存")),
                ("查看已安装包数量", "pacman -Q | wc -l", None),
                ("查看孤立包", "pacman -Qdt", None),
                ("删除孤立包", "pacman -Rns $(pacman -Qdtq) --noconfirm", Some("将移除所有不再被依赖的包")),
            ],
            PackageManager::Dnf => vec![
                ("刷新软件源", "dnf makecache", None),
                ("检查可更新包", "dnf check-update", None),
                ("升级系统", "dnf upgrade -y", Some("将升级所有软件包")),
                ("清理缓存", "dnf clean all", None),
                ("查看已安装包数量", "dnf list installed | wc -l", None),
                ("自动移除依赖", "dnf autoremove -y", Some("将删除不再需要的依赖包")),
            ],
            PackageManager::Apt => vec![
                ("刷新软件源", "apt update", None),
                ("检查可更新包", "apt list --upgradable 2>/dev/null", None),
                ("升级系统", "apt upgrade -y", Some("将升级所有软件包")),
                ("清理缓存", "apt clean", None),
                ("自动移除依赖", "apt autoremove -y", Some("将删除不再需要的依赖包")),
            ],
            PackageManager::Zypper => vec![
                ("刷新软件源", "zypper refresh", None),
                ("检查可更新包", "zypper list-updates", None),
                ("升级系统", "zypper update -y", Some("将升级所有软件包")),
                ("清理缓存", "zypper clean --all", None),
            ],
            PackageManager::Unknown => vec![],
        }
    }
}

pub struct PackageManagerModule {
    pub selected_index: usize,
    pkg_mgr: PackageManager,
    list: SelectableList,
}

impl PackageManagerModule {
    pub fn new() -> Self {
        let pkg_mgr = PackageManager::detect();
        let actions = pkg_mgr.actions();
        let items: Vec<String> = actions.iter().map(|(name, cmd, _)| {
            format!("{} ({})", name, cmd)
        }).collect();

        Self {
            selected_index: 0,
            pkg_mgr,
            list: SelectableList::new(items),
        }
    }
}

impl TuiModule for PackageManagerModule {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<CommandToExecute> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.list.previous();
                self.selected_index = self.list.selected_index();
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.list.next();
                self.selected_index = self.list.selected_index();
                None
            }
            KeyCode::Enter => {
                let actions = self.pkg_mgr.actions();
                let idx = self.list.selected_index();
                if idx < actions.len() {
                    let (name, cmd, warn) = actions[idx];
                    Some(CommandToExecute {
                        display_name: name.to_string(),
                        command_string: cmd.to_string(),
                        risk_warning: warn.map(|s| s.to_string()),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(5),    // List
                Constraint::Length(2), // Hint
            ])
            .split(area);

        // Header showing detected package manager
        let header_text = format!(
            "  检测到包管理器: {}  │  共 {} 个可用操作",
            self.pkg_mgr.name(),
            self.pkg_mgr.actions().len()
        );
        let header = Paragraph::new(Line::from(Span::styled(
            header_text,
            Style::default().fg(Color::Green),
        )))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        );
        frame.render_widget(header, chunks[0]);

        // Selectable list
        if self.pkg_mgr == PackageManager::Unknown {
            let msg = Paragraph::new("  ⚠ 未检测到已知的包管理器 (pacman/dnf/apt/zypper)")
                .style(Style::default().fg(Color::Red))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" 📦 软件包管理 ")
                        .border_style(Style::default().fg(Color::Blue)),
                );
            frame.render_widget(msg, chunks[1]);
        } else {
            self.list.render(frame, chunks[1], "📦 软件包管理");
        }

        let hint = Paragraph::new(Line::from(Span::styled(
            "  ↑↓ 选择  │  Enter 执行  │  所有操作执行前需确认",
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(hint, chunks[2]);
    }

    fn refresh(&mut self) {
        // Re-detect package manager in case environment changed
        let pkg_mgr = PackageManager::detect();
        if pkg_mgr != self.pkg_mgr {
            self.pkg_mgr = pkg_mgr;
            let actions = self.pkg_mgr.actions();
            let items: Vec<String> = actions.iter().map(|(name, cmd, _)| {
                format!("{} ({})", name, cmd)
            }).collect();
            self.list = SelectableList::new(items);
        }
    }
}
