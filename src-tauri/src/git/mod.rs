pub mod diff;
pub mod history;
pub mod repository;
pub mod watcher;

pub use diff::FileDiff;
pub use history::CommitInfo;
pub use watcher::WatcherManager;

use git2::Repository;
use parking_lot::RwLock;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

// Status types returned to frontend
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitStatus {
    pub branch: Option<String>,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub staged: Vec<FileStatus>,
    pub unstaged: Vec<FileStatus>,
    pub untracked: Vec<String>,
    pub conflicted: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileStatus {
    pub path: String,
    pub status: FileStatusType,
    pub old_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum FileStatusType {
    Modified,
    Added,
    Deleted,
    Renamed,
    #[allow(dead_code)] // Reserved for future use
    Copied,
    TypeChanged,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchInfo {
    pub name: String,
    pub is_head: bool,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
}

/// Manages git repository paths (repositories are opened on-demand for thread safety)
pub struct GitManager {
    repositories: Arc<RwLock<HashMap<String, PathBuf>>>,
}

impl GitManager {
    pub fn new() -> Self {
        Self {
            repositories: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Helper to open a repository by ID
    fn open_repo(&self, id: &str) -> Result<Repository, String> {
        let repos = self.repositories.read();
        let path = repos
            .get(id)
            .ok_or_else(|| format!("Repository {} not found", id))?;

        Repository::open(path).map_err(|e| format!("Failed to open repository: {}", e))
    }

    /// Discover a git repository from a given path
    pub fn discover_repository(path: &str) -> Result<String, String> {
        let repo = Repository::discover(path)
            .map_err(|e| format!("Failed to discover repository: {}", e))?;

        let workdir = repo
            .workdir()
            .ok_or_else(|| "Repository has no working directory".to_string())?;

        Ok(workdir.to_string_lossy().to_string())
    }

    /// Register a repository path and return an ID
    pub fn open_repository(&self, path: &str) -> Result<String, String> {
        // Verify it's a valid repo first
        Repository::open(path).map_err(|e| format!("Failed to open repository: {}", e))?;

        let id = uuid::Uuid::new_v4().to_string();
        self.repositories
            .write()
            .insert(id.clone(), PathBuf::from(path));
        Ok(id)
    }

    /// Remove a repository from tracking
    pub fn close_repository(&self, id: &str) -> Result<(), String> {
        self.repositories
            .write()
            .remove(id)
            .ok_or_else(|| format!("Repository {} not found", id))?;
        Ok(())
    }

    /// Get repository status
    pub fn get_status(&self, id: &str) -> Result<GitStatus, String> {
        let repo = self.open_repo(id)?;
        repository::get_repository_status(&repo)
    }

    /// Get current branch info
    pub fn get_branch_info(&self, id: &str) -> Result<BranchInfo, String> {
        let repo = self.open_repo(id)?;
        repository::get_current_branch(&repo)
    }

    /// List all branches
    pub fn list_branches(&self, id: &str) -> Result<Vec<BranchInfo>, String> {
        let repo = self.open_repo(id)?;
        repository::list_branches(&repo)
    }

    /// Stage a file
    pub fn stage_file(&self, id: &str, path: &str) -> Result<(), String> {
        let repo = self.open_repo(id)?;
        repository::stage_file(&repo, path)
    }

    /// Unstage a file
    pub fn unstage_file(&self, id: &str, path: &str) -> Result<(), String> {
        let repo = self.open_repo(id)?;
        repository::unstage_file(&repo, path)
    }

    /// Stage all changes
    pub fn stage_all(&self, id: &str) -> Result<(), String> {
        let repo = self.open_repo(id)?;
        repository::stage_all(&repo)
    }

    /// Create a commit
    pub fn commit(&self, id: &str, message: &str) -> Result<String, String> {
        let repo = self.open_repo(id)?;
        repository::create_commit(&repo, message)
    }

    /// Get diff for a specific file
    pub fn get_file_diff(&self, id: &str, path: &str, staged: bool) -> Result<FileDiff, String> {
        let repo = self.open_repo(id)?;
        diff::get_file_diff(&repo, path, staged)
    }

    /// Get diff for a file in a specific commit
    pub fn get_commit_file_diff(
        &self,
        id: &str,
        commit_id: &str,
        path: &str,
    ) -> Result<FileDiff, String> {
        let repo = self.open_repo(id)?;
        diff::get_commit_file_diff(&repo, commit_id, path)
    }

    /// Get diff stats (files changed, insertions, deletions)
    pub fn get_diff_stats(&self, id: &str, staged: bool) -> Result<(u32, u32, u32), String> {
        let repo = self.open_repo(id)?;
        diff::get_diff_stats(&repo, staged)
    }

    /// Fetch from remote
    pub fn fetch(&self, id: &str, remote: &str) -> Result<(), String> {
        let repo = self.open_repo(id)?;
        repository::fetch(&repo, remote)
    }

    /// Pull from remote
    pub fn pull(&self, id: &str, remote: &str) -> Result<String, String> {
        let repo = self.open_repo(id)?;
        repository::pull(&repo, remote)
    }

    /// Push to remote
    pub fn push(&self, id: &str, remote: &str) -> Result<(), String> {
        let repo = self.open_repo(id)?;
        repository::push(&repo, remote)
    }

    /// List remotes
    pub fn list_remotes(&self, id: &str) -> Result<Vec<String>, String> {
        let repo = self.open_repo(id)?;
        repository::list_remotes(&repo)
    }

    /// Get commit history
    pub fn get_commit_log(
        &self,
        id: &str,
        limit: usize,
        skip: usize,
    ) -> Result<Vec<CommitInfo>, String> {
        let repo = self.open_repo(id)?;
        history::get_commit_log(&repo, limit, skip)
    }

    /// Get single commit info
    pub fn get_commit(&self, id: &str, commit_id: &str) -> Result<CommitInfo, String> {
        let repo = self.open_repo(id)?;
        history::get_commit(&repo, commit_id)
    }

    /// Get files changed in a commit
    pub fn get_commit_files(&self, id: &str, commit_id: &str) -> Result<Vec<String>, String> {
        let repo = self.open_repo(id)?;
        history::get_commit_files(&repo, commit_id)
    }

    // ============ Branch Operations ============

    /// Checkout an existing branch
    pub fn checkout_branch(&self, id: &str, branch_name: &str) -> Result<(), String> {
        let repo = self.open_repo(id)?;
        repository::checkout_branch(&repo, branch_name)
    }

    /// Create a new branch
    pub fn create_branch(&self, id: &str, branch_name: &str, checkout: bool) -> Result<(), String> {
        let repo = self.open_repo(id)?;
        repository::create_branch(&repo, branch_name, checkout)
    }

    /// Delete a branch
    pub fn delete_branch(&self, id: &str, branch_name: &str) -> Result<(), String> {
        let repo = self.open_repo(id)?;
        repository::delete_branch(&repo, branch_name)
    }

    // ============ Discard Operations ============

    /// Discard changes in a single file
    pub fn discard_file(&self, id: &str, path: &str) -> Result<(), String> {
        let repo = self.open_repo(id)?;
        repository::discard_file(&repo, path)
    }

    /// Discard all unstaged changes
    pub fn discard_all_unstaged(&self, id: &str) -> Result<(), String> {
        let repo = self.open_repo(id)?;
        repository::discard_all_unstaged(&repo)
    }

    /// Clean untracked files
    pub fn clean_untracked(&self, id: &str, paths: Option<Vec<String>>) -> Result<u32, String> {
        let repo = self.open_repo(id)?;
        repository::clean_untracked(&repo, paths)
    }
}

impl Default for GitManager {
    fn default() -> Self {
        Self::new()
    }
}
