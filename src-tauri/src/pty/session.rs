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
    /// Current working directory (updated via OSC sequences or process queries)
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
        let shell = shell.unwrap_or_else(|| detect_default_shell());

        // Determine working directory - default to user home directory
        let cwd = cwd.unwrap_or_else(|| {
            dirs::home_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| {
                    std::env::current_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| "C:\\".to_string())
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

        // Build command - for PowerShell, inject a custom prompt that emits OSC 9;9 sequences
        let is_powershell = shell.to_lowercase().contains("powershell")
            || shell.to_lowercase().contains("pwsh");

        let mut cmd = if is_powershell {
            let mut c = CommandBuilder::new(&shell);
            c.arg("-NoExit");
            c.arg("-Command");
            // Define a prompt function that emits OSC 9;9 with current directory
            // OSC 9;9;path ST where ST is ESC \ (0x1b 0x5c)
            c.arg(r#"function prompt { $p = $executionContext.SessionState.Path.CurrentLocation.Path; $e = [char]27; "$e]9;9;$p$e\PS $p> " }"#);
            c
        } else {
            CommandBuilder::new(&shell)
        };

        cmd.cwd(&cwd);

        // Set environment variables for shell integration (OSC sequences)
        // This enables PowerShell 7.2+ to emit OSC 9;9 sequences with current directory
        cmd.env("TERM_PROGRAM", "MythicorTerminal");
        cmd.env("TERM_PROGRAM_VERSION", "0.1.0");

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

    /// Get the process ID of the shell
    #[allow(dead_code)] // Used by try_get_process_cwd for fallback CWD detection
    pub fn get_pid(&self) -> Option<u32> {
        self.child.lock().process_id()
    }

    /// Update the current working directory (called when OSC sequences are parsed)
    pub fn set_cwd(&self, cwd: String) {
        *self.current_cwd.lock() = cwd;
    }

    /// Get the current working directory
    /// Returns the tracked CWD (from OSC sequences) or falls back to initial_cwd
    pub fn get_cwd(&self) -> Result<String, String> {
        Ok(self.current_cwd.lock().clone())
    }

    /// Try to get CWD from process memory (Windows fallback)
    #[cfg(windows)]
    #[allow(dead_code)] // Reserved for fallback when OSC sequences unavailable
    pub fn try_get_process_cwd(&self) -> Option<String> {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::{
            OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
        };

        let pid = self.get_pid()?;

        unsafe {
            let process_handle = OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                false,
                pid,
            ).ok()?;

            let result = get_process_cwd_internal(process_handle);
            let _ = CloseHandle(process_handle);
            result.ok()
        }
    }

    #[cfg(not(windows))]
    #[allow(dead_code)] // Reserved for fallback when OSC sequences unavailable
    pub fn try_get_process_cwd(&self) -> Option<String> {
        // On Unix, read /proc/<pid>/cwd
        let pid = self.get_pid()?;
        std::fs::read_link(format!("/proc/{}/cwd", pid))
            .map(|p| p.to_string_lossy().to_string())
            .ok()
    }
}

#[cfg(windows)]
#[allow(dead_code)] // Used by try_get_process_cwd
unsafe fn get_process_cwd_internal(
    process_handle: windows::Win32::Foundation::HANDLE,
) -> Result<String, String> {
    use ntapi::ntpsapi::{NtQueryInformationProcess, ProcessBasicInformation, PROCESS_BASIC_INFORMATION};
    use ntapi::ntrtl::RTL_USER_PROCESS_PARAMETERS;
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;

    // Get PEB address
    let mut pbi: PROCESS_BASIC_INFORMATION = std::mem::zeroed();
    let mut return_length: u32 = 0;

    let status = NtQueryInformationProcess(
        process_handle.0 as *mut _,
        ProcessBasicInformation,
        &mut pbi as *mut _ as *mut _,
        std::mem::size_of::<PROCESS_BASIC_INFORMATION>() as u32,
        &mut return_length,
    );

    if status != 0 {
        return Err(format!("NtQueryInformationProcess failed: {:#x}", status));
    }

    // Read PEB
    #[repr(C)]
    struct PEB {
        reserved1: [u8; 2],
        being_debugged: u8,
        reserved2: [u8; 1],
        reserved3: [*mut std::ffi::c_void; 2],
        ldr: *mut std::ffi::c_void,
        process_parameters: *mut RTL_USER_PROCESS_PARAMETERS,
    }

    let mut peb: PEB = std::mem::zeroed();
    let mut bytes_read = 0usize;

    let result = ReadProcessMemory(
        process_handle,
        pbi.PebBaseAddress as *const _,
        &mut peb as *mut _ as *mut _,
        std::mem::size_of::<PEB>(),
        Some(&mut bytes_read),
    );

    if result.is_err() {
        return Err("Failed to read PEB".to_string());
    }

    // Read RTL_USER_PROCESS_PARAMETERS
    let mut params: RTL_USER_PROCESS_PARAMETERS = std::mem::zeroed();
    let result = ReadProcessMemory(
        process_handle,
        peb.process_parameters as *const _,
        &mut params as *mut _ as *mut _,
        std::mem::size_of::<RTL_USER_PROCESS_PARAMETERS>(),
        Some(&mut bytes_read),
    );

    if result.is_err() {
        return Err("Failed to read process parameters".to_string());
    }

    // Read CurrentDirectory string
    let cwd_length = params.CurrentDirectory.DosPath.Length as usize / 2;
    let mut cwd_buffer: Vec<u16> = vec![0; cwd_length];

    let result = ReadProcessMemory(
        process_handle,
        params.CurrentDirectory.DosPath.Buffer as *const _,
        cwd_buffer.as_mut_ptr() as *mut _,
        params.CurrentDirectory.DosPath.Length as usize,
        Some(&mut bytes_read),
    );

    if result.is_err() {
        return Err("Failed to read current directory".to_string());
    }

    let cwd = OsString::from_wide(&cwd_buffer);
    Ok(cwd.to_string_lossy().to_string())
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
    std::env::var("PATH").ok().and_then(|path| {
        for dir in path.split(';') {
            let full_path = std::path::Path::new(dir).join(name);
            if full_path.exists() {
                return Some(full_path.to_string_lossy().to_string());
            }
        }
        None
    })
}
