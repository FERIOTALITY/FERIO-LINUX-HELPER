use crate::commands::{CommandExecutor, CommandToExecute};
use crate::modules::TuiModule;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub struct LogViewerModule {
    log_content: Vec<String>,
    scroll_offset: usize,
    service_query: String,
    is_entering_service: bool,
    current_category: usize,
    loaded: bool,
}

impl LogViewerModule {
    pub fn new() -> Self {
        Self {
            log_content: Vec::new(),
            scroll_offset: 0,
            service_query: String::new(),
            is_entering_service: false,
            current_category: 0,
            loaded: false,
        }
    }

    fn load_log(&mut self, category: usize) {
        self.current_category = category;
        self.scroll_offset = 0;

        let cmd = match category {
            0 => "journalctl -p 3 -n 200 --no-pager 2>/dev/null".to_string(),
            1 => "journalctl -b -n 200 --no-pager 2>/dev/null".to_string(),
            2 => "journalctl -k -n 200 --no-pager 2>/dev/null".to_string(),
            3 => {
                if self.service_query.is_empty() {
                    self.log_content = vec!["请输入服务名后按 Enter 查询".to_string()];
                    return;
                }
                format!("journalctl -u {} -n 200 --no-pager 2>/dev/null", self.service_query)
            }
            _ => return,
        };

        match CommandExecutor::run_silent(&cmd) {
            Ok(output) => {
                if output.is_empty() {
                    self.log_content = vec!["（无日志条目）".to_string()];
                } else {
                    self.log_content = output.lines().map(|l| l.to_string()).collect();
                }
            }
            Err(e) => {
                self.log_content = vec![format!("加载日志失败: {}", e)];
            }
        }
    }
}

impl TuiModule for LogViewerModule {
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<CommandToExecute> {
        // Service name input mode
        if self.is_entering_service {
            match key.code {
                KeyCode::Esc => {
                    self.is_entering_service = false;
                }
                KeyCode::Backspace => {
                    self.service_query.pop();
                }
                KeyCode::Enter => {
                    self.is_entering_service = false;
                    self.load_log(3);
                }
                KeyCode::Char(c) => {
                    self.service_query.push(c);
                }
                _ => {}
            }
            return None;
        }

        match key.code {
            // Category selection (left pane conceptually)
            KeyCode::Char('1') => { self.load_log(0); }
            KeyCode::Char('2') => { self.load_log(1); }
            KeyCode::Char('3') => { self.load_log(2); }
            KeyCode::Char('4') => {
                self.is_entering_service = true;
            }
            // Scroll log content
            KeyCode::Up | KeyCode::Char('k') => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.scroll_offset < self.log_content.len().saturating_sub(1) {
                    self.scroll_offset += 1;
                }
            }
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(20);
            }
            KeyCode::PageDown => {
                self.scroll_offset = std::cmp::min(
                    self.scroll_offset + 20,
                    self.log_content.len().saturating_sub(1),
                );
            }
            KeyCode::Home => {
                self.scroll_offset = 0;
            }
            KeyCode::End => {
                self.scroll_offset = self.log_content.len().saturating_sub(1);
            }
            KeyCode::Char('r') => {
                self.load_log(self.current_category);
            }
            _ => {}
        }
        None
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Category selector
                Constraint::Min(5),    // Log content
                Constraint::Length(2), // Hint
            ])
            .split(area);

        // Category buttons
        let cat_text = if self.is_entering_service {
            format!("  输入服务名: {}_", self.service_query)
        } else {
            "  [1] 错误日志  [2] 启动日志  [3] 内核日志  [4] 指定服务".to_string()
        };
        let cat_style = if self.is_entering_service {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Cyan)
        };
        let cat_bar = Paragraph::new(Line::from(Span::styled(cat_text, cat_style)))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            );
        frame.render_widget(cat_bar, chunks[0]);

        // Log content with scroll
        let visible_height = chunks[1].height.saturating_sub(2) as usize;
        let end = std::cmp::min(self.scroll_offset + visible_height, self.log_content.len());
        let visible: Vec<Line> = self.log_content[self.scroll_offset..end]
            .iter()
            .map(|l| {
                let style = if l.contains("error") || l.contains("ERROR") || l.contains("failed") {
                    Style::default().fg(Color::Red)
                } else if l.contains("warning") || l.contains("WARNING") {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };
                Line::from(Span::styled(l.clone(), style))
            })
            .collect();

        let scroll_info = format!(
            " 📋 日志查看  [{}/{}] ",
            self.scroll_offset + 1,
            self.log_content.len()
        );
        let log_widget = Paragraph::new(visible)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(scroll_info)
                    .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                    .border_style(Style::default().fg(Color::Blue)),
            );
        frame.render_widget(log_widget, chunks[1]);

        let hint = Paragraph::new(Line::from(Span::styled(
            "  ↑↓/PgUp/PgDn 滚动  │  Home/End 首尾  │  R 刷新  │  1-4 切换类别",
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(hint, chunks[2]);
    }

    fn refresh(&mut self) {
        if !self.loaded {
            self.load_log(0);
            self.loaded = true;
        }
    }
}
