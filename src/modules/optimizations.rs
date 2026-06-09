use crate::commands::CommandToExecute;
use crate::modules::TuiModule;
use crate::tui::widgets::SelectableList;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

struct OptimizationEntry {
    name: &'static str,
    description: &'static str,
    command: &'static str,
    risk_warning: &'static str,
    risk_level: RiskLevel,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum RiskLevel {
    Low,
    Medium,
    High,
}

impl RiskLevel {
    fn label(&self) -> &str {
        match self {
            RiskLevel::Low => "低风险",
            RiskLevel::Medium => "中风险",
            RiskLevel::High => "高风险",
        }
    }

    fn color(&self) -> Color {
        match self {
            RiskLevel::Low => Color::Green,
            RiskLevel::Medium => Color::Yellow,
            RiskLevel::High => Color::Red,
        }
    }
}

const OPTIMIZATIONS: &[OptimizationEntry] = &[
    OptimizationEntry {
        name: "降低 Swappiness (10)",
        description: "将 vm.swappiness 设为 10，减少使用交换空间，优先使用物理内存。适合内存充足的桌面系统。",
        command: "sysctl -w vm.swappiness=10",
        risk_warning: "仅在当前会话生效，重启后恢复默认值。如需永久生效需修改 /etc/sysctl.conf",
        risk_level: RiskLevel::Low,
    },
    OptimizationEntry {
        name: "清理系统日志 (保留7天)",
        description: "使用 journalctl --vacuum-time=7d 清理超过 7 天的系统日志，释放磁盘空间。",
        command: "journalctl --vacuum-time=7d",
        risk_warning: "超过7天的日志将被永久删除",
        risk_level: RiskLevel::Low,
    },
    OptimizationEntry {
        name: "清理系统日志 (限制500MB)",
        description: "限制系统日志总大小不超过 500MB。",
        command: "journalctl --vacuum-size=500M",
        risk_warning: "多余日志将被永久删除",
        risk_level: RiskLevel::Low,
    },
    OptimizationEntry {
        name: "清理临时文件",
        description: "清理 /tmp 和用户缓存中的临时文件。",
        command: "rm -rf /tmp/* 2>/dev/null; rm -rf ~/.cache/tmp/* 2>/dev/null",
        risk_warning: "正在运行的程序可能依赖 /tmp 中的文件",
        risk_level: RiskLevel::Medium,
    },
    OptimizationEntry {
        name: "启用 fstrim (SSD优化)",
        description: "启用 fstrim.timer 定时 TRIM 服务，延长 SSD 寿命并维持性能。",
        command: "systemctl enable --now fstrim.timer",
        risk_warning: "仅适用于 SSD 存储设备，机械硬盘不需要",
        risk_level: RiskLevel::Low,
    },
    OptimizationEntry {
        name: "提高文件监视上限",
        description: "增加 inotify 文件监视数量上限到 524288，解决 VSCode 等编辑器的监视限制警告。",
        command: "sysctl -w fs.inotify.max_user_watches=524288",
        risk_warning: "仅当前会话生效。如需永久生效需写入 /etc/sysctl.d/",
        risk_level: RiskLevel::Low,
    },
    OptimizationEntry {
        name: "禁用 core dump",
        description: "禁用核心转储文件生成，节省磁盘空间。",
        command: "ulimit -c 0 && echo 'kernel.core_pattern=/dev/null' | tee /etc/sysctl.d/50-coredump.conf && sysctl -p /etc/sysctl.d/50-coredump.conf",
        risk_warning: "禁用后程序崩溃时将无法生成调试信息",
        risk_level: RiskLevel::Medium,
    },
    OptimizationEntry {
        name: "清理用户缓存",
        description: "清理 ~/.cache 下超过 30 天的缓存文件。",
        command: "find ~/.cache -type f -atime +30 -delete 2>/dev/null",
        risk_warning: "部分应用可能需要重新生成缓存",
        risk_level: RiskLevel::Low,
    },
];

pub struct OptimizationsModule {
    pub selected_index: usize,
    list: SelectableList,
}

impl OptimizationsModule {
    pub fn new() -> Self {
        let items: Vec<String> = OPTIMIZATIONS
            .iter()
            .map(|o| {
                let risk_tag = match o.risk_level {
                    RiskLevel::Low => "[低]",
                    RiskLevel::Medium => "[中]",
                    RiskLevel::High => "[高]",
                };
                format!("{} {}", risk_tag, o.name)
            })
            .collect();

        Self {
            selected_index: 0,
            list: SelectableList::new(items),
        }
    }
}

impl TuiModule for OptimizationsModule {
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
                if idx < OPTIMIZATIONS.len() {
                    let opt = &OPTIMIZATIONS[idx];
                    Some(CommandToExecute {
                        display_name: opt.name.to_string(),
                        command_string: opt.command.to_string(),
                        risk_warning: Some(format!("[{}] {}", opt.risk_level.label(), opt.risk_warning)),
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
                Constraint::Length(6), // Detail panel
                Constraint::Length(2), // Hint
            ])
            .split(area);

        // Optimization list
        self.list.render(frame, chunks[0], "🚀 一键优化");

        // Detail panel showing description, command, and risk
        let idx = self.list.selected_index();
        let detail = if idx < OPTIMIZATIONS.len() {
            let opt = &OPTIMIZATIONS[idx];
            let risk_color = opt.risk_level.color();
            vec![
                Line::from(Span::styled(
                    format!("  {}", opt.description),
                    Style::default().fg(Color::White),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  命令: ", Style::default().fg(Color::Gray)),
                    Span::styled(opt.command, Style::default().fg(Color::Green)),
                ]),
                Line::from(Span::styled(
                    format!("  ⚠ [{}] {}", opt.risk_level.label(), opt.risk_warning),
                    Style::default().fg(risk_color),
                )),
            ]
        } else {
            vec![]
        };

        let detail_widget = Paragraph::new(detail)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 详情 ")
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(detail_widget, chunks[1]);

        let hint = Paragraph::new(Line::from(Span::styled(
            "  ↑↓ 选择  │  Enter 执行  │  每个操作执行前需确认",
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(hint, chunks[2]);
    }

    fn refresh(&mut self) {}
}
