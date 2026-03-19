use super::config::Workspace;
use std::fs;
use std::path::PathBuf;

/// Get the workspaces directory
fn get_workspaces_dir() -> Result<PathBuf, String> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| "Could not find config directory".to_string())?;

    let workspaces_dir = config_dir.join("mythicor-terminal").join("workspaces");

    // Create directory if it doesn't exist
    if !workspaces_dir.exists() {
        fs::create_dir_all(&workspaces_dir)
            .map_err(|e| format!("Failed to create workspaces directory: {}", e))?;
    }

    Ok(workspaces_dir)
}

/// Get workspace file path
fn get_workspace_path(id: &str) -> Result<PathBuf, String> {
    let dir = get_workspaces_dir()?;
    Ok(dir.join(format!("{}.json", id)))
}

/// Save a workspace to disk
pub fn save_workspace(workspace: &Workspace) -> Result<(), String> {
    let path = get_workspace_path(&workspace.id)?;
    let json = serde_json::to_string_pretty(workspace)
        .map_err(|e| format!("Failed to serialize workspace: {}", e))?;

    fs::write(&path, json).map_err(|e| format!("Failed to write workspace file: {}", e))?;

    Ok(())
}

/// Load a workspace from disk
pub fn load_workspace(id: &str) -> Result<Workspace, String> {
    let path = get_workspace_path(id)?;

    if !path.exists() {
        return Err(format!("Workspace {} not found", id));
    }

    let json =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read workspace file: {}", e))?;

    let workspace: Workspace =
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse workspace: {}", e))?;

    Ok(workspace)
}

/// Delete a workspace from disk
pub fn delete_workspace(id: &str) -> Result<(), String> {
    let path = get_workspace_path(id)?;

    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Failed to delete workspace file: {}", e))?;
    }

    Ok(())
}

/// List all saved workspaces
pub fn list_workspaces() -> Result<Vec<Workspace>, String> {
    let dir = get_workspaces_dir()?;
    let mut workspaces = Vec::new();

    let entries =
        fs::read_dir(&dir).map_err(|e| format!("Failed to read workspaces directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.extension().map(|e| e == "json").unwrap_or(false) {
            if let Ok(json) = fs::read_to_string(&path) {
                if let Ok(workspace) = serde_json::from_str::<Workspace>(&json) {
                    workspaces.push(workspace);
                }
            }
        }
    }

    // Sort by updated_at descending (most recent first)
    workspaces.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    Ok(workspaces)
}

/// Check if a workspace exists
#[allow(dead_code)] // Available for validation before operations
pub fn workspace_exists(id: &str) -> Result<bool, String> {
    let path = get_workspace_path(id)?;
    Ok(path.exists())
}
