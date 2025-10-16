use std::env;
use std::io;
use std::process::{Command, Stdio};

// Function to check if a command is available in the system
pub fn is_command_available(cmd: &str) -> bool {
    Command::new("which") // Unix-like systems
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or_else(|_| {
            // On Windows, "which" isn't available, so try running the command directly
            Command::new(cmd)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|status| status.success())
                .unwrap_or(false)
        })
}

pub fn is_xcode_tools_installed() -> bool {
    Command::new("xcode-select")
        .arg("-p")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn get_user_home() -> io::Result<String> {
    // Try SUDO_USER first to get the invoking user's home directory
    if let Ok(sudo_user) = env::var("SUDO_USER") {
        Ok(format!("/Users/{}", sudo_user))
    } else {
        // Fall back to $HOME if not running with sudo
        env::var("HOME")
            .map_err(|e| io::Error::new(io::ErrorKind::NotFound, format!("Failed to get HOME: {}", e)))
    }
}

//capitalize the first letter in a string
pub fn capitalize_first(s: &str) -> String {
    match s.get(0..1) {
        None => String::new(),
        Some(first) => first.to_uppercase() + &s[1..],
    }
}