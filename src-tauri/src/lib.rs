mod git;
mod pty;
mod workspace;

use git::{BranchInfo, CommitInfo, FileDiff, GitManager, GitStatus, WatcherManager};
use parking_lot::RwLock;
use pty::{detect_shells, PtyManager, ShellInfo};
use std::io::Read;
use std::sync::Arc;
use std::thread;
use tauri::{AppHandle, Emitter, State};
use workspace::{delete_workspace, list_workspaces, load_workspace, save_workspace, Workspace};

/// Shared state for the application
struct AppState {
    pty_manager: PtyManager,
    git_manager: GitManager,
    watcher_manager: WatcherManager,
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
    let session_for_cwd = session.clone();

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

                    // Parse OSC sequences for CWD updates
                    // OSC 9;9;path ST (PowerShell/ConEmu style)
                    // OSC 7;file://host/path ST (macOS/Linux style)
                    if let Some(cwd) = parse_osc_cwd(&data) {
                        session_for_cwd.set_cwd(cwd);
                    }

                    let _ = app.emit(&format!("terminal-data-{}", session_id_clone), data);
                }
                Err(e) => {
                    eprintln!("Read error for session {}: {}", session_id_clone, e);
                    let _ = app.emit(
                        &format!("terminal-error-{}", session_id_clone),
                        e.to_string(),
                    );
                    break;
                }
            }
        }
    });

    Ok(())
}

/// Parse OSC escape sequences to extract current working directory
fn parse_osc_cwd(data: &str) -> Option<String> {
    // OSC = \x1b] (ESC ])
    // ST = \x1b\ (ESC \) or \x07 (BEL)

    // PowerShell/ConEmu style: OSC 9;9;path ST
    if let Some(start) = data.find("\x1b]9;9;") {
        let path_start = start + 6; // Skip "\x1b]9;9;"
        let remaining = &data[path_start..];

        // Find terminator (BEL or ESC \)
        let end = remaining
            .find('\x07')
            .or_else(|| remaining.find("\x1b\\"))
            .unwrap_or(remaining.len());

        let path = remaining[..end].trim();
        if !path.is_empty() {
            return Some(path.to_string());
        }
    }

    // macOS/Linux style: OSC 7;file://host/path ST
    if let Some(start) = data.find("\x1b]7;") {
        let url_start = start + 4; // Skip "\x1b]7;"
        let remaining = &data[url_start..];

        let end = remaining
            .find('\x07')
            .or_else(|| remaining.find("\x1b\\"))
            .unwrap_or(remaining.len());

        let url = remaining[..end].trim();
        if let Some(path) = url.strip_prefix("file://") {
            // Skip hostname (find first / after the host)
            if let Some(slash_pos) = path.find('/') {
                let path_part = &path[slash_pos..];
                // Basic URL decode for common cases (spaces as %20)
                let decoded = path_part.replace("%20", " ");
                return Some(decoded);
            }
        }
    }

    None
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

// ============ Git Commands ============

/// Discover a git repository from a path
#[tauri::command]
fn git_discover_repo(path: String) -> Result<String, String> {
    GitManager::discover_repository(&path)
}

/// Open a git repository
#[tauri::command]
fn git_open_repo(state: State<'_, Arc<RwLock<AppState>>>, path: String) -> Result<String, String> {
    state.read().git_manager.open_repository(&path)
}

/// Close a git repository
#[tauri::command]
fn git_close_repo(state: State<'_, Arc<RwLock<AppState>>>, repo_id: String) -> Result<(), String> {
    state.read().git_manager.close_repository(&repo_id)
}

/// Get repository status
#[tauri::command]
fn git_status(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
) -> Result<GitStatus, String> {
    state.read().git_manager.get_status(&repo_id)
}

/// Get current branch info
#[tauri::command]
fn git_branch_info(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
) -> Result<BranchInfo, String> {
    state.read().git_manager.get_branch_info(&repo_id)
}

/// List all branches
#[tauri::command]
fn git_list_branches(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
) -> Result<Vec<BranchInfo>, String> {
    state.read().git_manager.list_branches(&repo_id)
}

/// Stage a file
#[tauri::command]
fn git_stage_file(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    path: String,
) -> Result<(), String> {
    state.read().git_manager.stage_file(&repo_id, &path)
}

/// Unstage a file
#[tauri::command]
fn git_unstage_file(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    path: String,
) -> Result<(), String> {
    state.read().git_manager.unstage_file(&repo_id, &path)
}

/// Stage all changes
#[tauri::command]
fn git_stage_all(state: State<'_, Arc<RwLock<AppState>>>, repo_id: String) -> Result<(), String> {
    state.read().git_manager.stage_all(&repo_id)
}

/// Create a commit
#[tauri::command]
fn git_commit(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    message: String,
) -> Result<String, String> {
    state.read().git_manager.commit(&repo_id, &message)
}

/// Get diff for a specific file
#[tauri::command]
fn git_diff_file(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    path: String,
    staged: bool,
) -> Result<FileDiff, String> {
    state
        .read()
        .git_manager
        .get_file_diff(&repo_id, &path, staged)
}

/// Get diff for a file from a specific commit
#[tauri::command]
fn git_diff_commit(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    commit_id: String,
    path: String,
) -> Result<FileDiff, String> {
    state
        .read()
        .git_manager
        .get_commit_file_diff(&repo_id, &commit_id, &path)
}

/// Get diff stats (files changed, insertions, deletions)
#[tauri::command]
fn git_diff_stats(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    staged: bool,
) -> Result<(u32, u32, u32), String> {
    state.read().git_manager.get_diff_stats(&repo_id, staged)
}

/// Fetch from remote
#[tauri::command]
fn git_fetch(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    remote: Option<String>,
) -> Result<(), String> {
    let remote = remote.unwrap_or_else(|| "origin".to_string());
    state.read().git_manager.fetch(&repo_id, &remote)
}

/// Pull from remote
#[tauri::command]
fn git_pull(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    remote: Option<String>,
) -> Result<String, String> {
    let remote = remote.unwrap_or_else(|| "origin".to_string());
    state.read().git_manager.pull(&repo_id, &remote)
}

/// Push to remote
#[tauri::command]
fn git_push(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    remote: Option<String>,
    set_upstream: Option<bool>,
) -> Result<(), String> {
    let remote = remote.unwrap_or_else(|| "origin".to_string());
    let set_upstream = set_upstream.unwrap_or(false);
    state.read().git_manager.push(&repo_id, &remote, set_upstream)
}

/// List remotes
#[tauri::command]
fn git_list_remotes(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
) -> Result<Vec<String>, String> {
    state.read().git_manager.list_remotes(&repo_id)
}

/// Get commit history
#[tauri::command]
fn git_log(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    limit: Option<usize>,
    skip: Option<usize>,
) -> Result<Vec<CommitInfo>, String> {
    let limit = limit.unwrap_or(50);
    let skip = skip.unwrap_or(0);
    state
        .read()
        .git_manager
        .get_commit_log(&repo_id, limit, skip)
}

/// Get single commit info
#[tauri::command]
fn git_show_commit(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    commit_id: String,
) -> Result<CommitInfo, String> {
    state.read().git_manager.get_commit(&repo_id, &commit_id)
}

/// Get files changed in a commit
#[tauri::command]
fn git_commit_files(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    commit_id: String,
) -> Result<Vec<String>, String> {
    state
        .read()
        .git_manager
        .get_commit_files(&repo_id, &commit_id)
}

/// Start watching a repository for file changes
#[tauri::command]
fn git_start_watcher(
    app: AppHandle,
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    repo_path: String,
) -> Result<(), String> {
    state
        .read()
        .watcher_manager
        .start_watching(app, repo_id, std::path::PathBuf::from(repo_path))
}

/// Stop watching a repository
#[tauri::command]
fn git_stop_watcher(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
) -> Result<(), String> {
    state.read().watcher_manager.stop_watching(&repo_id);
    Ok(())
}

// ============ Branch Commands ============

/// Checkout an existing branch
#[tauri::command]
fn git_checkout_branch(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    branch_name: String,
) -> Result<(), String> {
    state
        .read()
        .git_manager
        .checkout_branch(&repo_id, &branch_name)
}

/// Create a new branch
#[tauri::command]
fn git_create_branch(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    branch_name: String,
    checkout: Option<bool>,
) -> Result<(), String> {
    let checkout = checkout.unwrap_or(true);
    state
        .read()
        .git_manager
        .create_branch(&repo_id, &branch_name, checkout)
}

/// Delete a branch
#[tauri::command]
fn git_delete_branch(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    branch_name: String,
) -> Result<(), String> {
    state
        .read()
        .git_manager
        .delete_branch(&repo_id, &branch_name)
}

// ============ Discard Commands ============

/// Discard changes in a single file
#[tauri::command]
fn git_discard_file(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    path: String,
) -> Result<(), String> {
    state.read().git_manager.discard_file(&repo_id, &path)
}

/// Discard all unstaged changes
#[tauri::command]
fn git_discard_all(state: State<'_, Arc<RwLock<AppState>>>, repo_id: String) -> Result<(), String> {
    state.read().git_manager.discard_all_unstaged(&repo_id)
}

/// Clean untracked files
#[tauri::command]
fn git_clean_untracked(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    paths: Option<Vec<String>>,
) -> Result<u32, String> {
    state.read().git_manager.clean_untracked(&repo_id, paths)
}

// ============ Clipboard Commands ============

/// Save clipboard image data to a temp file and return the path
#[tauri::command]
fn save_clipboard_image(rgba_data: Vec<u8>, width: u32, height: u32) -> Result<String, String> {
    use image::{ImageBuffer, Rgba};
    use std::time::{SystemTime, UNIX_EPOCH};

    // Create temp directory for clipboard images
    let temp_dir = std::env::temp_dir().join("primarch-clipboard");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;

    // Generate filename with timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let filename = format!("clipboard_{}.png", timestamp);
    let file_path = temp_dir.join(&filename);

    // Create image from RGBA data
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, rgba_data)
            .ok_or_else(|| "Failed to create image from RGBA data".to_string())?;

    // Save as PNG
    img.save(&file_path)
        .map_err(|e| format!("Failed to save image: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

// ============ AI Commands ============

/// Generate a commit message from staged changes using the Anthropic API
#[tauri::command]
async fn generate_commit_message(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    api_key: String,
) -> Result<String, String> {
    // Use compact diff for API to minimize token usage (and cost)
    let diff_text = state.read().git_manager.get_staged_diff_compact(&repo_id)?;

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "model": "claude-haiku-4-5-20251001",
            "max_tokens": 300,
            "messages": [{
                "role": "user",
                "content": format!(
                    "Generate a concise git commit message for the following staged diff. \
                     Use conventional commit format (e.g., feat:, fix:, refactor:). \
                     First line should be under 72 characters. Add a blank line and brief \
                     body only if the changes are complex. Do not include any explanation \
                     outside the commit message itself. Do not wrap the message in backticks \
                     or any markdown formatting - output only the raw commit message text.\n\n\
                     Diff:\n{}\n",
                    diff_text
                )
            }]
        }))
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    let status = response.status();
    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    if !status.is_success() {
        let error_msg = body["error"]["message"]
            .as_str()
            .unwrap_or("Unknown API error");
        return Err(format!("API error: {}", error_msg));
    }

    body["content"][0]["text"]
        .as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| "No content in API response".to_string())
}

/// Resolve the user's full PATH by sourcing their shell config.
/// On macOS, GUI apps get a minimal PATH missing ~/.local/bin, nvm, cargo, etc.
#[cfg(not(windows))]
pub(crate) fn resolve_user_path() -> String {
    use std::sync::OnceLock;
    static CACHED_PATH: OnceLock<String> = OnceLock::new();

    CACHED_PATH
        .get_or_init(|| {
            // Use login interactive shell so both .zprofile AND .zshrc are sourced.
            // Redirect stderr to /dev/null to suppress any prompt/config noise.
            if let Ok(output) = std::process::Command::new("/bin/zsh")
                .args(["-l", "-i", "-c", "echo $PATH"])
                .stderr(std::process::Stdio::null())
                .output()
            {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() {
                        return path;
                    }
                }
            }

            // Fallback: append common user paths to current PATH
            let current = std::env::var("PATH").unwrap_or_default();
            if let Some(home) = dirs::home_dir() {
                let home = home.to_string_lossy();
                let extras = [
                    format!("{}/.local/bin", home),
                    format!("{}/.cargo/bin", home),
                    "/opt/homebrew/bin".to_string(),
                    "/opt/homebrew/sbin".to_string(),
                ];
                let mut parts: Vec<&str> = current.split(':').collect();
                for extra in &extras {
                    if !parts.contains(&extra.as_str()) {
                        parts.push(extra);
                    }
                }
                return parts.join(":");
            }

            current
        })
        .clone()
}

#[derive(serde::Serialize)]
struct DirEntry {
    name: String,
    path: String,
    is_dir: bool,
}

#[tauri::command]
fn list_directory(path: String) -> Result<Vec<DirEntry>, String> {
    let resolved = if path.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            home.join(&path[1..].trim_start_matches('/'))
        } else {
            std::path::PathBuf::from(&path)
        }
    } else {
        std::path::PathBuf::from(&path)
    };

    let mut entries = Vec::new();
    let read_dir = std::fs::read_dir(&resolved)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in read_dir.take(200) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let name = entry.file_name().to_string_lossy().to_string();
        // Skip hidden dirs in the listing
        if name.starts_with('.') {
            continue;
        }
        if metadata.is_dir() {
            entries.push(DirEntry {
                name,
                path: entry.path().to_string_lossy().to_string(),
                is_dir: true,
            });
        }
    }

    entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(entries)
}

#[tauri::command]
async fn search_directories(query: String) -> Result<Vec<DirEntry>, String> {
    if query.len() < 2 {
        return Ok(vec![]);
    }

    // Use Spotlight on macOS for instant indexed search
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("mdfind")
            .args([
                "-name",
                &query,
                "-onlyin",
                &dirs::home_dir()
                    .map(|h| h.to_string_lossy().to_string())
                    .unwrap_or_else(|| "/Users".to_string()),
            ])
            .output()
            .map_err(|e| format!("mdfind failed: {}", e))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let entries: Vec<DirEntry> = stdout
                .lines()
                .filter(|line| {
                    let path = std::path::Path::new(line);
                    path.is_dir()
                        && !line.contains("/Library/")
                        && !line.contains("/.Trash/")
                        && !line.contains("/node_modules/")
                        && !line.contains("/.git/")
                        && !line.contains("/target/")
                })
                .take(50)
                .map(|line| {
                    let name = std::path::Path::new(line)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    DirEntry {
                        name,
                        path: line.to_string(),
                        is_dir: true,
                    }
                })
                .collect();
            return Ok(entries);
        }
    }

    // Fallback for non-macOS: simple recursive search from home (depth-limited)
    #[cfg(not(target_os = "macos"))]
    {
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"));
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        fn walk(dir: &std::path::Path, query: &str, results: &mut Vec<DirEntry>, depth: u32) {
            if depth > 4 || results.len() >= 50 {
                return;
            }
            let read_dir = match std::fs::read_dir(dir) {
                Ok(rd) => rd,
                Err(_) => return,
            };
            for entry in read_dir {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') || name == "node_modules" || name == "target" {
                    continue;
                }
                if let Ok(meta) = entry.metadata() {
                    if meta.is_dir() {
                        if name.to_lowercase().contains(query) {
                            results.push(DirEntry {
                                name: name.clone(),
                                path: entry.path().to_string_lossy().to_string(),
                                is_dir: true,
                            });
                        }
                        walk(&entry.path(), query, results, depth + 1);
                    }
                }
            }
        }

        walk(&home, &query_lower, &mut results, 0);
        results.truncate(50);
        return Ok(results);
    }

    #[allow(unreachable_code)]
    Ok(vec![])
}

/// Check if a CLI tool is available (Windows-compatible)
fn is_cli_available(name: &str) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        std::process::Command::new("cmd")
            .args(["/C", name, "--version"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        // On macOS, GUI apps have a minimal PATH. Use a login interactive
        // shell to resolve the full PATH (needs -i so ~/.zshrc is sourced).
        let full_path = resolve_user_path();

        let mut cmd = std::process::Command::new(name);
        cmd.arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        if !full_path.is_empty() {
            cmd.env("PATH", &full_path);
        }

        cmd.status().map(|s| s.success()).unwrap_or(false)
    }
}

/// Detect available AI CLI tools
#[tauri::command]
fn detect_ai_clis() -> Vec<String> {
    let mut available = Vec::new();

    if is_cli_available("claude") {
        available.push("claude".to_string());
    }

    if is_cli_available("codex") {
        available.push("codex".to_string());
    }

    available
}

/// Generate a commit message using an AI CLI tool (claude, codex, etc.)
#[tauri::command]
async fn generate_commit_message_cli(
    state: State<'_, Arc<RwLock<AppState>>>,
    repo_id: String,
    cli: String,
) -> Result<String, String> {
    let diff_text = state.read().git_manager.get_staged_diff_text(&repo_id)?;

    if diff_text.trim().is_empty() {
        return Err("No staged changes to generate commit message from".to_string());
    }

    let prompt = format!(
        "Generate a concise git commit message for the following staged diff. \
         Use conventional commit format (e.g., feat:, fix:, refactor:). \
         First line should be under 72 characters. Add a blank line and brief \
         body only if the changes are complex. Do not include any explanation \
         outside the commit message itself. Do not wrap the message in backticks \
         or any markdown formatting - output only the raw commit message text.\n\n\
         Diff:\n{}\n",
        diff_text
    );

    let output = match cli.as_str() {
        "claude" | "codex" => {
            use std::process::{Command, Stdio};

            // Write prompt to a temp file to avoid command line length/escaping issues
            let temp_dir = std::env::temp_dir();
            let temp_file = temp_dir.join(format!("primarch_prompt_{}.txt", std::process::id()));

            std::fs::write(&temp_file, &prompt)
                .map_err(|e| format!("Failed to write prompt to temp file: {}", e))?;

            // Run from temp directory to avoid CLI picking up local git context
            // --tools '' disables tool access so it only uses our prompt
            #[cfg(windows)]
            let result = {
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                let ps_command = format!(
                    "Set-Location '{}'; Get-Content -Raw '{}' | {} --tools '' -p -",
                    temp_dir.to_string_lossy().replace("'", "''"),
                    temp_file.to_string_lossy().replace("'", "''"),
                    cli
                );
                Command::new("powershell")
                    .args(["-NoProfile", "-Command", &ps_command])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .creation_flags(CREATE_NO_WINDOW)
                    .output()
            };

            #[cfg(not(windows))]
            let result = {
                let shell_command = format!(
                    "cd '{}' && cat '{}' | {} --tools '' -p -",
                    temp_dir.to_string_lossy().replace("'", "'\\''"),
                    temp_file.to_string_lossy().replace("'", "'\\''"),
                    cli
                );
                let mut cmd = Command::new("sh");
                cmd.args(["-c", &shell_command])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());

                // Ensure full user PATH is available on macOS
                #[cfg(target_os = "macos")]
                {
                    let full_path = resolve_user_path();
                    if !full_path.is_empty() {
                        cmd.env("PATH", &full_path);
                    }
                }

                cmd.output()
            };

            // Clean up temp file
            let _ = std::fs::remove_file(&temp_file);

            result.map_err(|e| format!("Failed to run {} CLI: {}", cli, e))?
        }
        _ => return Err(format!("Unknown CLI: {}", cli)),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("CLI error: {}", stderr));
    }

    let message = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if message.is_empty() {
        return Err("CLI returned empty response".to_string());
    }

    Ok(message)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(RwLock::new(AppState {
        pty_manager: PtyManager::new(),
        git_manager: GitManager::new(),
        watcher_manager: WatcherManager::new(),
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        // Emit event to frontend
                        let _ = app.emit("global-shortcut", shortcut.to_string());
                    }
                })
                .build(),
        )
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            // Terminal commands
            create_terminal,
            write_terminal,
            resize_terminal,
            kill_terminal,
            list_terminals,
            get_terminal_cwd,
            start_terminal_reader,
            get_available_shells,
            // Workspace commands
            save_workspace_cmd,
            load_workspace_cmd,
            delete_workspace_cmd,
            list_workspaces_cmd,
            // Git commands
            git_discover_repo,
            git_open_repo,
            git_close_repo,
            git_status,
            git_branch_info,
            git_list_branches,
            git_stage_file,
            git_unstage_file,
            git_stage_all,
            git_commit,
            git_diff_file,
            git_diff_commit,
            git_diff_stats,
            git_fetch,
            git_pull,
            git_push,
            git_list_remotes,
            git_log,
            git_show_commit,
            git_commit_files,
            git_start_watcher,
            git_stop_watcher,
            // Branch commands
            git_checkout_branch,
            git_create_branch,
            git_delete_branch,
            // Discard commands
            git_discard_file,
            git_discard_all,
            git_clean_untracked,
            // Clipboard commands
            save_clipboard_image,
            // AI commands
            generate_commit_message,
            generate_commit_message_cli,
            detect_ai_clis,
            // Directory commands
            list_directory,
            search_directories,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
