use serde::{Deserialize, Serialize};
use std::process::Command;

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
    PowerShell,
    Cmd,
    Wsl,
    Git,
    Other,
}

/// Detect all available shells on the system
pub fn detect_shells() -> Vec<ShellInfo> {
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

/// Detect PowerShell Core (pwsh)
fn detect_pwsh() -> Option<ShellInfo> {
    // Check if pwsh exists in PATH
    let output = Command::new("where").arg("pwsh.exe").output().ok()?;

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

/// Detect Windows PowerShell
fn detect_windows_powershell() -> Option<ShellInfo> {
    let output = Command::new("where")
        .arg("powershell.exe")
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

/// Detect Git Bash
fn detect_git_bash() -> Option<ShellInfo> {
    // Common Git Bash locations
    let paths = [
        r"C:\Program Files\Git\bin\bash.exe",
        r"C:\Program Files (x86)\Git\bin\bash.exe",
    ];

    for path in paths {
        if std::path::Path::new(path).exists() {
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

/// Detect WSL distributions
fn detect_wsl_distros() -> Vec<ShellInfo> {
    let mut distros = Vec::new();

    // Run wsl --list --quiet to get distribution names
    let output = Command::new("wsl.exe")
        .args(["--list", "--quiet"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);

            for line in stdout.lines() {
                // Clean up the line (WSL output can have BOM and other artifacts)
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

    // Add default WSL if any distros were found
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

/// Get the default shell
#[allow(dead_code)] // Available for programmatic shell selection
pub fn get_default_shell() -> ShellInfo {
    let shells = detect_shells();
    shells.into_iter().next().unwrap_or_else(|| ShellInfo {
        id: "powershell".to_string(),
        name: "Windows PowerShell".to_string(),
        command: "powershell.exe".to_string(),
        args: vec![],
        shell_type: ShellType::PowerShell,
    })
}
