use clap::Parser;
use crossterm::event::{self, Event, KeyEventKind};
use std::env;
use std::time::Duration;

mod commands;
mod modules;
mod privilege;
mod tui;

use tui::app::App;
use tui::TerminalGuard;

#[derive(Parser, Debug)]
#[command(
    name = "ferio-linux-helper",
    version,
    about = "A TUI Linux desktop configuration and troubleshooting assistant"
)]
struct Cli {
    /// Enable dry-run/mock mode: commands are logged but never executed
    #[arg(long)]
    dry_run: bool,

    /// Skip root privilege check (useful for viewing system info only)
    #[arg(long)]
    no_root: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    // Privilege auto-escalation check (skip if --no-root)
    if !args.no_root {
        privilege::verify_and_escalate()?;
    }

    // Determine if dry-run or mock execution is active
    let dry_run = args.dry_run || env::var("MOCK_EXEC").is_ok();

    // Initialize alternate screen & raw mode with panic recovery
    let mut guard = TerminalGuard::init()?;
    let mut app = App::new(dry_run);

    // Initial data load
    app.refresh_active_module();

    // Main TUI event loop
    while !app.should_quit {
        guard.terminal.draw(|f| {
            app.draw(f);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    app.handle_key_event(key_event);
                }
            }
        }

        app.tick();
    }

    // Restore terminal
    drop(guard);

    // Output dry-run log summary on exit
    if dry_run && !app.executor.history.is_empty() {
        println!("\n--- Dry-Run Command Log ---");
        for (i, cmd) in app.executor.history.iter().enumerate() {
            println!("  [{}] {}", i + 1, cmd);
        }
        println!("--- End Log ---\n");
    }

    Ok(())
}
