use crate::commands::CommandToExecute;
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::Frame;

/// Common trait for all TUI modules.
pub trait TuiModule {
    /// Handles keyboard events. Returns Some(CommandToExecute) if a system action is triggered.
    fn handle_key_event(&mut self, key: KeyEvent) -> Option<CommandToExecute>;

    /// Renders the module's UI within the given area.
    fn draw(&mut self, frame: &mut Frame, area: Rect);

    /// Called periodically for background data updates.
    #[allow(dead_code)]
    fn update(&mut self) {}

    /// Called when module becomes active (tab switched to it).
    fn refresh(&mut self) {}
}

pub mod desktop_fixes;
pub mod log_viewer;
pub mod network;
pub mod optimizations;
pub mod package_manager;
pub mod system_info;
pub mod systemd;
