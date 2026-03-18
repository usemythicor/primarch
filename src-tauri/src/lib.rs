mod pty;
mod workspace;

use parking_lot::RwLock;
use pty::{detect_shells, PtyManager, ShellInfo};
use std::io::Read;
use std::sync::Arc;
use std::thread;
use tauri::{AppHandle, Emitter, State};
use workspace::{delete_workspace, list_workspaces, load_workspace, save_workspace, Workspace};

/// Shared state for the PTY manager
struct AppState {
    pty_manager: PtyManager,
}

/// Create a new terminal session
#[tauri::command]
fn create_terminal(
    state: State<'_, Arc<RwLock<AppState>>>,
    shell: Option<String>,
    cwd: Option<String>,
) -> Result<String, String> {
    state.read().pty_manager.create_session(shell, cwd)
}

/// Write data to a terminal session
#[tauri::command]
fn write_terminal(
    state: State<'_, Arc<RwLock<AppState>>>,
    session_id: String,
    data: String,
) -> Result<(), String> {
    state
        .read()
        .pty_manager
        .write_to_session(&session_id, data.as_bytes())
}

/// Resize a terminal session
#[tauri::command]
fn resize_terminal(
    state: State<'_, Arc<RwLock<AppState>>>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    state
        .read()
        .pty_manager
        .resize_session(&session_id, cols, rows)
}

/// Kill a terminal session
#[tauri::command]
fn kill_terminal(
    state: State<'_, Arc<RwLock<AppState>>>,
    session_id: String,
) -> Result<(), String> {
    state.read().pty_manager.kill_session(&session_id)
}

/// List all terminal sessions
#[tauri::command]
fn list_terminals(state: State<'_, Arc<RwLock<AppState>>>) -> Vec<String> {
    state.read().pty_manager.list_sessions()
}

/// Get the current working directory of a terminal session
#[tauri::command]
fn get_terminal_cwd(
    state: State<'_, Arc<RwLock<AppState>>>,
    session_id: String,
) -> Result<String, String> {
    state.read().pty_manager.get_session_cwd(&session_id)
}

/// Start reading from a terminal and emit events
#[tauri::command]
fn start_terminal_reader(
    app: AppHandle,
    state: State<'_, Arc<RwLock<AppState>>>,
    session_id: String,
) -> Result<(), String> {
    let session = state
        .read()
        .pty_manager
        .get_session(&session_id)
        .ok_or_else(|| format!("Session {} not found", session_id))?;

    let reader = session.get_reader();
    let session_id_clone = session_id.clone();

    // Spawn a thread to read from the PTY and emit events
    thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.lock().read(&mut buf) {
                Ok(0) => {
                    // EOF - terminal closed
                    let _ = app.emit(&format!("terminal-closed-{}", session_id_clone), ());
                    break;
                }
                Ok(n) => {
                    // Convert to string and emit
                    let data = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = app.emit(&format!("terminal-data-{}", session_id_clone), data);
                }
                Err(e) => {
                    eprintln!("Read error for session {}: {}", session_id_clone, e);
                    let _ = app.emit(&format!("terminal-error-{}", session_id_clone), e.to_string());
                    break;
                }
            }
        }
    });

    Ok(())
}

/// Get all available shells on the system
#[tauri::command]
fn get_available_shells() -> Vec<ShellInfo> {
    detect_shells()
}

// ============ Workspace Commands ============

/// Save a workspace
#[tauri::command]
fn save_workspace_cmd(workspace: Workspace) -> Result<(), String> {
    save_workspace(&workspace)
}

/// Load a workspace by ID
#[tauri::command]
fn load_workspace_cmd(id: String) -> Result<Workspace, String> {
    load_workspace(&id)
}

/// Delete a workspace
#[tauri::command]
fn delete_workspace_cmd(id: String) -> Result<(), String> {
    delete_workspace(&id)
}

/// List all workspaces
#[tauri::command]
fn list_workspaces_cmd() -> Result<Vec<Workspace>, String> {
    list_workspaces()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(RwLock::new(AppState {
        pty_manager: PtyManager::new(),
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            create_terminal,
            write_terminal,
            resize_terminal,
            kill_terminal,
            list_terminals,
            get_terminal_cwd,
            start_terminal_reader,
            get_available_shells,
            save_workspace_cmd,
            load_workspace_cmd,
            delete_workspace_cmd,
            list_workspaces_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
