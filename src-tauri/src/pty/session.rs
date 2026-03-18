use parking_lot::Mutex;
use portable_pty::{native_pty_system, Child, CommandBuilder, PtyPair, PtySize};
use std::io::{Read, Write};
use std::sync::Arc;

/// Represents a single terminal session
#[derive(Clone)]
pub struct TerminalSession {
    pub id: String,
    pub shell: String,
    pub initial_cwd: String,
    pty_pair: Arc<Mutex<PtyPair>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    reader: Arc<Mutex<Box<dyn Read + Send>>>,
    child: Arc<Mutex<Box<dyn Child + Send + Sync>>>,
}

impl TerminalSession {
    /// Create a new terminal session
    pub fn new(shell: Option<String>, cwd: Option<String>) -> Result<Self, String> {
        let id = uuid::Uuid::new_v4().to_string();

        // Determine shell to use
        let shell = shell.unwrap_or_else(|| detect_default_shell());

        // Determine working directory
        let cwd = cwd.unwrap_or_else(|| {
            std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| {
                    dirs::home_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| "C:\\".to_string())
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

        // Build command
        let mut cmd = CommandBuilder::new(&shell);
        cmd.cwd(&cwd);

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
            initial_cwd: cwd,
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
    pub fn get_pid(&self) -> Option<u32> {
        self.child.lock().process_id()
    }

    /// Get the current working directory of the shell process
    #[cfg(windows)]
    pub fn get_cwd(&self) -> Result<String, String> {
        use std::mem;
        use std::ptr;
        use windows::Win32::Foundation::{CloseHandle, HANDLE};
        use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
        use windows::Win32::System::Threading::{
            OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
        };

        let pid = self.get_pid().ok_or("Process ID not available")?;

        unsafe {
            // Open the process
            let process_handle = OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                false,
                pid,
            )
            .map_err(|e| format!("Failed to open process: {}", e))?;

            // Use ntapi to get process info
            let result = get_process_cwd_internal(process_handle);

            let _ = CloseHandle(process_handle);

            result
        }
    }

    #[cfg(not(windows))]
    pub fn get_cwd(&self) -> Result<String, String> {
        // On Unix, read /proc/<pid>/cwd
        if let Some(pid) = self.get_pid() {
            std::fs::read_link(format!("/proc/{}/cwd", pid))
                .map(|p| p.to_string_lossy().to_string())
                .map_err(|e| format!("Failed to read cwd: {}", e))
        } else {
            Ok(self.initial_cwd.clone())
        }
    }
}

#[cfg(windows)]
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
