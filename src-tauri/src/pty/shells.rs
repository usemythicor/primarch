use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Represents a detected shell on the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellInfo {
    pub id: String,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub shell_type: ShellType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ShellType {
    Zsh,
    Bash,
    Fish,
    PowerShell,
    Cmd,
    Wsl,
    Git,
    Other,
}

/// Detect all available shells on the system
pub fn detect_shells() -> Vec<ShellInfo> {
    #[cfg(windows)]
    {
        detect_shells_windows()
    }

    #[cfg(not(windows))]
    {
        detect_shells_unix()
    }
}

// ============================================================================
// Windows Shell Detection
// ============================================================================

#[cfg(windows)]
fn detect_shells_windows() -> Vec<ShellInfo> {
    let mut shells = Vec::new();

    // PowerShell Core (pwsh)
    if let Some(shell) = detect_pwsh() {
        shells.push(shell);
    }

    // Windows PowerShell
    if let Some(shell) = detect_windows_powershell() {
        shells.push(shell);
    }

    // Command Prompt
    shells.push(ShellInfo {
        id: "cmd".to_string(),
        name: "Command Prompt".to_string(),
        command: "cmd.exe".to_string(),
        args: vec![],
        shell_type: ShellType::Cmd,
    });

    // Git Bash
    if let Some(shell) = detect_git_bash() {
        shells.push(shell);
    }

    // WSL distributions
    shells.extend(detect_wsl_distros());

    shells
}

#[cfg(windows)]
fn detect_pwsh() -> Option<ShellInfo> {
    let output = Command::new("where")
        .arg("pwsh.exe")
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;

    if output.status.success() {
        Some(ShellInfo {
            id: "pwsh".to_string(),
            name: "PowerShell".to_string(),
            command: "pwsh.exe".to_string(),
            args: vec!["-NoLogo".to_string()],
            shell_type: ShellType::PowerShell,
        })
    } else {
        None
    }
}

#[cfg(windows)]
fn detect_windows_powershell() -> Option<ShellInfo> {
    let output = Command::new("where")
        .arg("powershell.exe")
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;

    if output.status.success() {
        Some(ShellInfo {
            id: "powershell".to_string(),
            name: "Windows PowerShell".to_string(),
            command: "powershell.exe".to_string(),
            args: vec!["-NoLogo".to_string()],
            shell_type: ShellType::PowerShell,
        })
    } else {
        None
    }
}

#[cfg(windows)]
fn detect_git_bash() -> Option<ShellInfo> {
    let paths = [
        r"C:\Program Files\Git\bin\bash.exe",
        r"C:\Program Files (x86)\Git\bin\bash.exe",
    ];

    for path in paths {
        if Path::new(path).exists() {
            return Some(ShellInfo {
                id: "git-bash".to_string(),
                name: "Git Bash".to_string(),
                command: path.to_string(),
                args: vec!["--login".to_string(), "-i".to_string()],
                shell_type: ShellType::Git,
            });
        }
    }

    None
}

#[cfg(windows)]
fn detect_wsl_distros() -> Vec<ShellInfo> {
    let mut distros = Vec::new();

    let output = Command::new("wsl.exe")
        .args(["--list", "--quiet"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);

            for line in stdout.lines() {
                let distro = line
                    .trim()
                    .trim_start_matches('\u{feff}')
                    .trim_matches(char::from(0))
                    .to_string();

                if !distro.is_empty() && !distro.contains("Windows Subsystem") {
                    distros.push(ShellInfo {
                        id: format!("wsl-{}", distro.to_lowercase().replace(' ', "-")),
                        name: format!("WSL: {}", distro),
                        command: "wsl.exe".to_string(),
                        args: vec!["-d".to_string(), distro.clone()],
                        shell_type: ShellType::Wsl,
                    });
                }
            }
        }
    }

    if !distros.is_empty() {
        distros.insert(
            0,
            ShellInfo {
                id: "wsl".to_string(),
                name: "WSL (Default)".to_string(),
                command: "wsl.exe".to_string(),
                args: vec![],
                shell_type: ShellType::Wsl,
            },
        );
    }

    distros
}

// ============================================================================
// Unix/macOS Shell Detection
// ============================================================================

#[cfg(not(windows))]
fn detect_shells_unix() -> Vec<ShellInfo> {
    let mut shells = Vec::new();

    // Get user's default shell from environment
    if let Ok(default_shell) = std::env::var("SHELL") {
        let shell_name = Path::new(&default_shell)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("shell");

        let shell_type = match shell_name {
            "zsh" => ShellType::Zsh,
            "bash" => ShellType::Bash,
            "fish" => ShellType::Fish,
            _ => ShellType::Other,
        };

        shells.push(ShellInfo {
            id: format!("{}-default", shell_name),
            name: format!("{} (Default)", capitalize(shell_name)),
            command: default_shell.clone(),
            args: vec!["--login".to_string()],
            shell_type,
        });
    }

    // Detect common shells
    let shell_configs = [
        ("/bin/zsh", "zsh", "Zsh", ShellType::Zsh),
        ("/usr/bin/zsh", "zsh", "Zsh", ShellType::Zsh),
        ("/bin/bash", "bash", "Bash", ShellType::Bash),
        ("/usr/bin/bash", "bash", "Bash", ShellType::Bash),
        ("/usr/local/bin/fish", "fish", "Fish", ShellType::Fish),
        ("/opt/homebrew/bin/fish", "fish", "Fish", ShellType::Fish),
        ("/usr/bin/fish", "fish", "Fish", ShellType::Fish),
        ("/bin/sh", "sh", "sh", ShellType::Other),
    ];

    for (path, id, name, shell_type) in shell_configs {
        // Skip if already added as default
        if shells.iter().any(|s| s.command == path) {
            continue;
        }

        if Path::new(path).exists() {
            shells.push(ShellInfo {
                id: id.to_string(),
                name: name.to_string(),
                command: path.to_string(),
                args: vec!["--login".to_string()],
                shell_type,
            });
        }
    }

    // Check for PowerShell Core on macOS/Linux
    if let Some(pwsh) = detect_pwsh_unix() {
        shells.push(pwsh);
    }

    shells
}

#[cfg(not(windows))]
fn detect_pwsh_unix() -> Option<ShellInfo> {
    let paths = [
        "/usr/local/bin/pwsh",
        "/opt/homebrew/bin/pwsh",
        "/usr/bin/pwsh",
    ];

    for path in paths {
        if Path::new(path).exists() {
            return Some(ShellInfo {
                id: "pwsh".to_string(),
                name: "PowerShell".to_string(),
                command: path.to_string(),
                args: vec!["-NoLogo".to_string()],
                shell_type: ShellType::PowerShell,
            });
        }
    }

    // Also check PATH
    if Command::new("which")
        .arg("pwsh")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some(ShellInfo {
            id: "pwsh".to_string(),
            name: "PowerShell".to_string(),
            command: "pwsh".to_string(),
            args: vec!["-NoLogo".to_string()],
            shell_type: ShellType::PowerShell,
        });
    }

    None
}

#[cfg(not(windows))]
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().chain(chars).collect(),
    }
}

// ============================================================================
// Common Functions
// ============================================================================

/// Get the default shell
#[allow(dead_code)] // Available for programmatic shell selection
pub fn get_default_shell() -> ShellInfo {
    let shells = detect_shells();

    #[cfg(windows)]
    let fallback = ShellInfo {
        id: "powershell".to_string(),
        name: "Windows PowerShell".to_string(),
        command: "powershell.exe".to_string(),
        args: vec![],
        shell_type: ShellType::PowerShell,
    };

    #[cfg(not(windows))]
    let fallback = ShellInfo {
        id: "bash".to_string(),
        name: "Bash".to_string(),
        command: "/bin/bash".to_string(),
        args: vec!["--login".to_string()],
        shell_type: ShellType::Bash,
    };

    shells.into_iter().next().unwrap_or(fallback)
}
