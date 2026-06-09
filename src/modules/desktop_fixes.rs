use crate::commands::CommandToExecute;
use crate::modules::TuiModule;
use crate::tui::widgets::SelectableList;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

struct FixEntry {
    name: &'static str,
    description: &'static str,
    command: &'static str,
    risk_warning: Option<&'static str>,
}

const FIXES: &[FixEntry] = &[
    FixEntry {
        name: "重启 KDE Plasma Shell",
        description: "终止并重启 plasmashell 进程，修复桌面面板和小组件问题",
        command: "kquitapp5 plasmashell || killall plasmashell; sleep 1; kstart5 plasmashell &",
        risk_warning: Some("桌面面板会短暂消失后重新加载"),

    },
    FixEntry {
        name: "重启 GNOME Shell",
        description: "通过 D-Bus 重启 GNOME Shell（仅 X11）",
        command: "busctl --user call org.gnome.Shell /org/gnome/Shell org.gnome.Shell Eval s 'Meta.restart(\"Restarting…\")' 2>/dev/null || gnome-shell --replace &",
        risk_warning: Some("GNOME Shell 会短暂闪烁"),

    },
    FixEntry {
        name: "重启用户级 systemd 服务",
        description: "重启当前用户的所有 systemd 用户级服务",
        command: "systemctl --user daemon-reload && systemctl --user restart --all",
        risk_warning: Some("所有用户级服务会短暂重启，可能影响正在运行的应用"),

    },
    FixEntry {
        name: "清理缩略图缓存",
        description: "删除 ~/.cache/thumbnails 下的所有缩略图缓存文件",
        command: "rm -rf ~/.cache/thumbnails/*",
        risk_warning: None,

    },
    FixEntry {
        name: "清理图标缓存",
        description: "重建图标缓存，修复图标显示异常",
        command: "gtk-update-icon-cache -f /usr/share/icons/hicolor 2>/dev/null; update-icon-caches /usr/share/icons/* 2>/dev/null",
        risk_warning: None,

    },
    FixEntry {
        name: "重置 KDE 面板配置",
        description: "删除 KDE 面板配置文件并重启 plasmashell（将恢复默认面板布局）",
        command: "mv ~/.config/plasma-org.kde.plasma.desktop-appletsrc ~/.config/plasma-org.kde.plasma.desktop-appletsrc.bak && kquitapp5 plasmashell; sleep 1; kstart5 plasmashell &",
        risk_warning: Some("当前面板布局将丢失，原配置已备份为 .bak 文件"),

    },
    FixEntry {
        name: "重启 PipeWire 音频",
        description: "重启 PipeWire 和 WirePlumber 音频服务",
        command: "systemctl --user restart pipewire pipewire-pulse wireplumber 2>/dev/null",
        risk_warning: Some("音频会短暂中断"),

    },
    FixEntry {
        name: "查看当前图形会话信息",
        description: "显示当前的显示服务器、桌面环境和会话类型",
        command: "echo \"Session: $XDG_SESSION_TYPE\"; echo \"Desktop: $XDG_CURRENT_DESKTOP\"; echo \"Display: $DISPLAY\"; echo \"Wayland: $WAYLAND_DISPLAY\"; loginctl show-session $(loginctl | grep $(whoami) | awk '{print $1}') -p Type -p Class -p Service 2>/dev/null",
        risk_warning: None,

    },
];

pub struct DesktopFixesModule {
    pub selected_index: usize,
    list: SelectableList,
    current_de: String,
}

impl DesktopFixesModule {
    pub fn new() -> Self {
        let de = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_lowercase();
        let items: Vec<String> = FIXES.iter().map(|f| f.name.to_string()).collect();

        Self {
            selected_index: 0,
            list: SelectableList::new(items),
            current_de: de,
        }
    }
}

impl TuiModule for DesktopFixesModule {
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
                let idx = self.list.selected_index();
                if idx < FIXES.len() {
                    let fix = &FIXES[idx];
                    Some(CommandToExecute {
                        display_name: fix.name.to_string(),
                        command_string: fix.command.to_string(),
                        risk_warning: fix.risk_warning.map(|s| s.to_string()),
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
                Constraint::Min(8),    // List
                Constraint::Length(5), // Description panel
                Constraint::Length(2), // Hint
            ])
            .split(area);

        // Fix list
        self.list.render(frame, chunks[0], "🔧 桌面环境修复");

        // Description of selected item
        let idx = self.list.selected_index();
        let desc = if idx < FIXES.len() {
            let fix = &FIXES[idx];
            let mut text = format!("  {}\n  命令: {}", fix.description, fix.command);
            if let Some(warn) = fix.risk_warning {
                text.push_str(&format!("\n  ⚠ {}", warn));
            }
            text
        } else {
            String::new()
        };

        let desc_widget = Paragraph::new(desc)
            .style(Style::default().fg(Color::Gray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 说明 ")
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .wrap(ratatui::widgets::Wrap { trim: false });
        frame.render_widget(desc_widget, chunks[1]);

        let de_info = if self.current_de.is_empty() {
            "未检测到桌面环境".to_string()
        } else {
            format!("当前桌面: {}", self.current_de.to_uppercase())
        };
        let hint = Paragraph::new(Line::from(Span::styled(
            format!("  {}  │  ↑↓ 选择  │  Enter 执行", de_info),
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(hint, chunks[2]);
    }

    fn refresh(&mut self) {
        self.current_de = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_lowercase();
    }
}
