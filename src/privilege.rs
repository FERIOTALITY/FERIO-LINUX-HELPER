use std::env;
use std::process::{self, Command};
use nix::unistd::Uid;

/// Returns true if the application is running with root/superuser privileges (UID 0).
/// Respects MOCK_ROOT / MOCK_NON_ROOT env vars for testing.
pub fn is_root() -> bool {
    if env::var("MOCK_NON_ROOT").is_ok() {
        return false;
    }
    if env::var("MOCK_ROOT").is_ok() {
        return true;
    }
    Uid::current().is_root()
}

/// Verifies whether the process runs with root privileges and handles auto-escalation.
/// If not root, relaunches the binary via `sudo -E` preserving environment.
pub fn verify_and_escalate() -> Result<(), std::io::Error> {
    if is_root() {
        return Ok(());
    }

    eprintln!("⚠  Root privileges required. Relaunching with sudo...");

    // In test/mock mode, exit with special code instead of launching sudo
    if env::var("MOCK_NON_ROOT").is_ok() {
        process::exit(100);
    }

    let current_exe = env::current_exe()?;
    let args: Vec<String> = env::args().skip(1).collect();

    let mut child = Command::new("sudo")
        .arg("-E")
        .arg(&current_exe)
        .args(&args)
        .spawn()?;

    let status = child.wait()?;
    let exit_code = status.code().unwrap_or(1);
    process::exit(exit_code);
}
