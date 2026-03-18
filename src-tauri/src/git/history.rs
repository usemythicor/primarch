use git2::{Repository, Revwalk, Sort};
use serde::Serialize;
use std::collections::HashMap;

/// Commit information for display
#[derive(Debug, Clone, Serialize)]
pub struct CommitInfo {
    pub oid: String,
    pub short_id: String,
    pub message: String,
    pub summary: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: i64,
    pub parent_ids: Vec<String>,
    pub refs: Vec<RefInfo>,
}

/// Reference information (branch, tag, etc.)
#[derive(Debug, Clone, Serialize)]
pub struct RefInfo {
    pub name: String,
    pub ref_type: RefType,
    pub is_head: bool,
}

#[derive(Debug, Clone, Serialize)]
pub enum RefType {
    Branch,
    RemoteBranch,
    Tag,
}

/// Get commit history with pagination
pub fn get_commit_log(
    repo: &Repository,
    limit: usize,
    skip: usize,
) -> Result<Vec<CommitInfo>, String> {
    let mut revwalk = repo.revwalk()
        .map_err(|e| format!("Failed to create revwalk: {}", e))?;

    // Start from HEAD
    revwalk.push_head()
        .map_err(|e| format!("Failed to push HEAD: {}", e))?;

    // Sort by time, newest first
    revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL)
        .map_err(|e| format!("Failed to set sorting: {}", e))?;

    // Build ref map for decorations
    let ref_map = build_ref_map(repo)?;

    let mut commits = Vec::new();
    let mut count = 0;

    for oid_result in revwalk {
        let oid = oid_result.map_err(|e| format!("Failed to get oid: {}", e))?;

        // Skip if before our window
        if count < skip {
            count += 1;
            continue;
        }

        // Stop if we've collected enough
        if commits.len() >= limit {
            break;
        }

        let commit = repo.find_commit(oid)
            .map_err(|e| format!("Failed to find commit: {}", e))?;

        let oid_str = oid.to_string();
        let refs = ref_map.get(&oid_str).cloned().unwrap_or_default();

        commits.push(CommitInfo {
            oid: oid_str.clone(),
            short_id: oid_str[..7.min(oid_str.len())].to_string(),
            message: commit.message().unwrap_or("").to_string(),
            summary: commit.summary().unwrap_or("").to_string(),
            author_name: commit.author().name().unwrap_or("Unknown").to_string(),
            author_email: commit.author().email().unwrap_or("").to_string(),
            timestamp: commit.time().seconds(),
            parent_ids: commit.parent_ids().map(|id| id.to_string()).collect(),
            refs,
        });

        count += 1;
    }

    Ok(commits)
}

/// Build a map from commit OID to refs pointing at it
fn build_ref_map(repo: &Repository) -> Result<HashMap<String, Vec<RefInfo>>, String> {
    let mut ref_map: HashMap<String, Vec<RefInfo>> = HashMap::new();

    let head = repo.head().ok();
    let head_name = head.as_ref()
        .and_then(|h| h.shorthand())
        .map(|s| s.to_string());

    // Iterate over all references
    let refs = repo.references()
        .map_err(|e| format!("Failed to list references: {}", e))?;

    for reference_result in refs {
        let reference = match reference_result {
            Ok(r) => r,
            Err(_) => continue,
        };

        // Get the target commit
        let target = match reference.peel_to_commit() {
            Ok(c) => c.id().to_string(),
            Err(_) => continue,
        };

        let ref_name = match reference.shorthand() {
            Some(n) => n.to_string(),
            None => continue,
        };

        let full_name = match reference.name() {
            Some(n) => n,
            None => continue,
        };

        let (ref_type, display_name) = if full_name.starts_with("refs/heads/") {
            (RefType::Branch, ref_name.clone())
        } else if full_name.starts_with("refs/remotes/") {
            (RefType::RemoteBranch, ref_name.clone())
        } else if full_name.starts_with("refs/tags/") {
            (RefType::Tag, ref_name.clone())
        } else {
            continue;
        };

        let is_head = Some(&ref_name) == head_name.as_ref();

        ref_map.entry(target).or_default().push(RefInfo {
            name: display_name,
            ref_type,
            is_head,
        });
    }

    Ok(ref_map)
}

/// Get a single commit's full info
pub fn get_commit(repo: &Repository, oid_str: &str) -> Result<CommitInfo, String> {
    let oid = git2::Oid::from_str(oid_str)
        .map_err(|e| format!("Invalid commit ID: {}", e))?;

    let commit = repo.find_commit(oid)
        .map_err(|e| format!("Commit not found: {}", e))?;

    let ref_map = build_ref_map(repo)?;
    let refs = ref_map.get(oid_str).cloned().unwrap_or_default();

    // Extract values to avoid borrow checker issues
    let message = commit.message().unwrap_or("").to_string();
    let summary = commit.summary().unwrap_or("").to_string();
    let author = commit.author();
    let author_name = author.name().unwrap_or("Unknown").to_string();
    let author_email = author.email().unwrap_or("").to_string();
    let timestamp = commit.time().seconds();
    let parent_ids: Vec<String> = commit.parent_ids().map(|id| id.to_string()).collect();

    Ok(CommitInfo {
        oid: oid_str.to_string(),
        short_id: oid_str[..7.min(oid_str.len())].to_string(),
        message,
        summary,
        author_name,
        author_email,
        timestamp,
        parent_ids,
        refs,
    })
}

/// Get files changed in a commit
pub fn get_commit_files(repo: &Repository, oid_str: &str) -> Result<Vec<String>, String> {
    let oid = git2::Oid::from_str(oid_str)
        .map_err(|e| format!("Invalid commit ID: {}", e))?;

    let commit = repo.find_commit(oid)
        .map_err(|e| format!("Commit not found: {}", e))?;

    let tree = commit.tree()
        .map_err(|e| format!("Failed to get commit tree: {}", e))?;

    let parent_tree = commit.parent(0)
        .ok()
        .and_then(|p| p.tree().ok());

    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)
        .map_err(|e| format!("Failed to get diff: {}", e))?;

    let mut files = Vec::new();

    diff.foreach(
        &mut |delta, _| {
            if let Some(path) = delta.new_file().path() {
                files.push(path.to_string_lossy().to_string());
            }
            true
        },
        None,
        None,
        None,
    ).map_err(|e| format!("Failed to iterate diff: {}", e))?;

    Ok(files)
}
