use parking_lot::Mutex;
use portable_pty::{native_pty_system, Child, CommandBuilder, PtyPair, PtySize};
use std::io::{Read, Write};
use std::sync::Arc;

/// Represents a single terminal session
#[derive(Clone)]
pub struct TerminalSession {
    pub id: String,
    #[allow(dead_code)] // Retained for debugging and future shell info display
    pub shell: String,
    #[allow(dead_code)] // Retained for workspace restoration
    pub initial_cwd: String,
    /// Current working directory (updated via OSC sequences)
    current_cwd: Arc<Mutex<String>>,
    pty_pair: Arc<Mutex<PtyPair>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    reader: Arc<Mutex<Box<dyn Read + Send>>>,
    #[allow(dead_code)] // Kept alive to maintain process lifecycle
    child: Arc<Mutex<Box<dyn Child + Send + Sync>>>,
}

impl TerminalSession {
    /// Create a new terminal session
    pub fn new(shell: Option<String>, cwd: Option<String>) -> Result<Self, String> {
        let id = uuid::Uuid::new_v4().to_string();

        // Determine shell to use
        let shell = shell.unwrap_or_else(detect_default_shell);

        // Determine working directory - default to user home directory.
        // Final fallback differs per platform so the PTY never tries to
        // chdir into a path that doesn't exist.
        let cwd = cwd.unwrap_or_else(|| {
            dirs::home_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| {
                    std::env::current_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| {
                            if cfg!(windows) {
                                "C:\\".to_string()
                            } else {
                                "/".to_string()
                            }
                        })
                })
        });

        // Create PTY system
        let pty_system = native_pty_system();

        // Create PTY pair with initial size
        let pty_pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to create PTY: {}", e))?;

        // Build command - inject shell integration for CWD reporting via OSC sequences
        let shell_lower = shell.to_lowercase();
        let is_powershell = shell_lower.contains("powershell") || shell_lower.contains("pwsh");
        let is_zsh = !is_powershell && shell_lower.contains("zsh");
        let is_bash = !is_powershell && !is_zsh && shell_lower.contains("bash");

        let mut cmd = if is_powershell {
            let mut c = CommandBuilder::new(&shell);
            c.arg("-NoExit");
            c.arg("-Command");
            // Define a prompt function that emits OSC 9;9 with current directory
            // OSC 9;9;path ST where ST is ESC \ (0x1b 0x5c)
            c.arg(r#"function prompt { $p = $executionContext.SessionState.Path.CurrentLocation.Path; $e = [char]27; "$e]9;9;$p$e\PS $p> " }"#);
            c
        } else if is_zsh {
            let mut c = CommandBuilder::new(&shell);
            c.arg("-l"); // Login shell: sources .zprofile for PATH setup (Homebrew, etc.)

            // Create a temporary ZDOTDIR with a .zshenv that:
            // 1. Restores the original ZDOTDIR so user configs load normally
            // 2. Adds a precmd hook to emit OSC 7 with the current directory
            let zdotdir = std::env::temp_dir().join("primarch-zsh-init");
            let _ = std::fs::create_dir_all(&zdotdir);
            let zshenv = r#"# Primarch shell integration
if [[ -n "$PRIMARCH_ORIG_ZDOTDIR" ]]; then
    ZDOTDIR="$PRIMARCH_ORIG_ZDOTDIR"
else
    ZDOTDIR="$HOME"
fi
unset PRIMARCH_ORIG_ZDOTDIR
[[ -f "$ZDOTDIR/.zshenv" ]] && source "$ZDOTDIR/.zshenv"
__primarch_precmd() { printf '\e]7;file://%s%s\e\\' "$HOST" "$PWD" }
precmd_functions+=(__primarch_precmd)
"#;
            let _ = std::fs::write(zdotdir.join(".zshenv"), zshenv);

            let orig_zdotdir = std::env::var("ZDOTDIR").unwrap_or_default();
            c.env("PRIMARCH_ORIG_ZDOTDIR", &orig_zdotdir);
            c.env("ZDOTDIR", zdotdir.to_string_lossy().as_ref());

            c
        } else if is_bash {
            let mut c = CommandBuilder::new(&shell);
            // Set PROMPT_COMMAND to emit OSC 7 with current directory
            c.env(
                "PROMPT_COMMAND",
                r#"printf '\e]7;file://%s%s\e\\' "$HOSTNAME" "$PWD""#,
            );
            c
        } else {
            CommandBuilder::new(&shell)
        };

        cmd.cwd(&cwd);

        // Set environment variables for shell integration (OSC sequences)
        // This enables PowerShell 7.2+ to emit OSC 9;9 sequences with current directory
        cmd.env("TERM_PROGRAM", "Primarch");
        cmd.env("TERM_PROGRAM_VERSION", "0.1.0");

        // Set TERM so the shell knows terminal capabilities (fixes delete key, etc.)
        cmd.env("TERM", "xterm-256color");

        // On macOS, GUI apps get a minimal PATH missing user paths like
        // ~/.local/bin, ~/.cargo/bin, nvm, homebrew, etc. Always resolve
        // the full login shell PATH and pass it to the PTY.
        #[cfg(target_os = "macos")]
        {
            let full_path = crate::resolve_user_path();
            if !full_path.is_empty() {
                cmd.env("PATH", &full_path);
            }
        }

        // Spawn the shell
        let child = pty_pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn shell: {}", e))?;

        // Get writer and reader
        let writer = pty_pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to get writer: {}", e))?;

        let reader = pty_pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to get reader: {}", e))?;

        Ok(Self {
            id,
            shell,
            initial_cwd: cwd.clone(),
            current_cwd: Arc::new(Mutex::new(cwd)),
            pty_pair: Arc::new(Mutex::new(pty_pair)),
            writer: Arc::new(Mutex::new(writer)),
            reader: Arc::new(Mutex::new(reader)),
            child: Arc::new(Mutex::new(child)),
        })
    }

    /// Write data to the terminal's stdin
    pub fn write(&self, data: &[u8]) -> Result<(), String> {
        self.writer
            .lock()
            .write_all(data)
            .map_err(|e| format!("Write error: {}", e))
    }

    /// Read data from the terminal's stdout
    #[allow(dead_code)] // Available for direct reading, currently using get_reader() instead
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, String> {
        self.reader
            .lock()
            .read(buf)
            .map_err(|e| format!("Read error: {}", e))
    }

    /// Resize the terminal
    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), String> {
        self.pty_pair
            .lock()
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Resize error: {}", e))
    }

    /// Kill the terminal process
    pub fn kill(&mut self) -> Result<(), String> {
        // The process will be killed when the PTY is dropped
        Ok(())
    }

    /// Get a clone of the reader for async reading
    pub fn get_reader(&self) -> Arc<Mutex<Box<dyn Read + Send>>> {
        self.reader.clone()
    }

    /// Update the current working directory (called when OSC sequences are parsed)
    pub fn set_cwd(&self, cwd: String) {
        *self.current_cwd.lock() = cwd;
    }

    /// Get the current working directory.
    /// Returns the tracked CWD — starts as initial_cwd and is kept
    /// up-to-date by OSC 7/9 sequences emitted by the shell's prompt hook.
    pub fn get_cwd(&self) -> Result<String, String> {
        Ok(self.current_cwd.lock().clone())
    }
}

/// Detect the default shell on the system
fn detect_default_shell() -> String {
    // On Windows, prefer PowerShell
    if cfg!(windows) {
        // Try to find pwsh (PowerShell Core) first, then fall back to powershell
        if which_shell("pwsh.exe").is_some() {
            return "pwsh.exe".to_string();
        }
        return "powershell.exe".to_string();
    }

    // On Unix, use SHELL env var or default to bash
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
}

/// Check if a shell exists in PATH
fn which_shell(name: &str) -> Option<String> {
    let path_separator = if cfg!(windows) { ';' } else { ':' };

    std::env::var("PATH").ok().and_then(|path| {
        for dir in path.split(path_separator) {
            let full_path = std::path::Path::new(dir).join(name);
            if full_path.exists() {
                return Some(full_path.to_string_lossy().to_string());
            }
        }
        None
    })
}
