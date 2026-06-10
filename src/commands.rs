use std::fs::OpenOptions;
use std::io::Write;
use std::process::Command;

/// Represents a system command that needs user confirmation before execution.
#[derive(Debug, Clone)]
pub struct CommandToExecute {
    /// Human-readable name shown in the confirmation dialog title
    pub display_name: String,
    /// The exact shell command string to execute
    pub command_string: String,
    /// Optional risk/warning text displayed in the confirmation modal
    pub risk_warning: Option<String>,
}

/// Result of a command execution
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub output: String,
}

/// Manages command execution with dry-run support and history tracking.
pub struct CommandExecutor {
    #[allow(dead_code)]
    pub dry_run: bool,
    pub history: Vec<String>,
    pub last_result: Option<CommandResult>,
}

impl CommandExecutor {
    pub fn new(dry_run: bool) -> Self {
        Self {
            dry_run,
            history: Vec::new(),
            last_result: None,
        }
    }

    /// Executes a command statically. Useful for running in background threads.
    pub fn execute_static(dry_run: bool, cmd: &CommandToExecute) -> Result<String, String> {
        let cmd_str = &cmd.command_string;
        if dry_run {
            // Log to file
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open("dry_run.log")
            {
                let _ = writeln!(file, "[DRY-RUN] {}: {}", cmd.display_name, cmd_str);
            }
            let msg = format!("[DRY-RUN] Would execute: {}", cmd_str);
            Ok(msg)
        } else {
            if cmd_str.trim().is_empty() {
                return Err("Empty command string".to_string());
            }
            match Command::new("sh").arg("-c").arg(cmd_str).output() {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
                    if output.status.success() {
                        if stdout.is_empty() {
                            Ok("Command completed successfully.".to_string())
                        } else {
                            Ok(stdout)
                        }
                    } else {
                        let err_msg = if stderr.is_empty() {
                            format!("Command failed with exit code: {}", output.status)
                        } else {
                            stderr
                        };
                        Err(err_msg)
                    }
                }
                Err(e) => {
                    Err(format!("Failed to spawn command: {}", e))
                }
            }
        }
    }

    /// Executes a command. In dry-run mode, logs to dry_run.log and returns mock success.
    #[allow(dead_code)]
    pub fn execute(&mut self, cmd: &CommandToExecute) -> Result<String, String> {
        self.history.push(cmd.command_string.clone());
        let res = Self::execute_static(self.dry_run, cmd);
        match &res {
            Ok(msg) => {
                self.last_result = Some(CommandResult {
                    success: true,
                    output: msg.clone(),
                });
            }
            Err(err) => {
                self.last_result = Some(CommandResult {
                    success: false,
                    output: err.clone(),
                });
            }
        }
        res
    }

    /// Run a command silently and return its stdout (used for data gathering, not user actions).
    pub fn run_silent(cmd_str: &str) -> Result<String, String> {
        match Command::new("sh").arg("-c").arg(cmd_str).output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
                }
            }
            Err(e) => Err(format!("Failed to run '{}': {}", cmd_str, e)),
        }
    }
}
