use git2::{Repository, DiffOptions, DiffFormat};
use serde::Serialize;

/// Represents a complete file diff
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDiff {
    pub path: String,
    pub old_path: Option<String>,
    pub hunks: Vec<DiffHunkData>,
    pub is_binary: bool,
    pub additions: u32,
    pub deletions: u32,
}

/// Represents a single hunk in a diff
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffHunkData {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub header: String,
    pub lines: Vec<DiffLineData>,
}

/// Represents a single line in a diff
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffLineData {
    pub origin: char,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub content: String,
}

/// Get diff for a specific file (staged or unstaged)
pub fn get_file_diff(repo: &Repository, path: &str, staged: bool) -> Result<FileDiff, String> {
    let mut diff_opts = DiffOptions::new();
    diff_opts.pathspec(path);
    diff_opts.context_lines(3);

    let diff = if staged {
        // Staged diff: HEAD vs Index
        let head_tree = repo.head()
            .ok()
            .and_then(|h| h.peel_to_tree().ok());

        repo.diff_tree_to_index(
            head_tree.as_ref(),
            None,
            Some(&mut diff_opts)
        ).map_err(|e| format!("Failed to get staged diff: {}", e))?
    } else {
        // Unstaged diff: Index vs Workdir
        repo.diff_index_to_workdir(
            None,
            Some(&mut diff_opts)
        ).map_err(|e| format!("Failed to get unstaged diff: {}", e))?
    };

    // Parse the diff into our structure
    let mut file_diff = FileDiff {
        path: path.to_string(),
        old_path: None,
        hunks: Vec::new(),
        is_binary: false,
        additions: 0,
        deletions: 0,
    };

    let mut current_hunk: Option<DiffHunkData> = None;

    diff.print(DiffFormat::Patch, |delta, hunk, line| {
        // Check for binary
        if delta.flags().is_binary() {
            file_diff.is_binary = true;
            return true;
        }

        // Handle old path for renames
        if let Some(old_file) = delta.old_file().path() {
            let old_path_str = old_file.to_string_lossy().to_string();
            if old_path_str != path {
                file_diff.old_path = Some(old_path_str);
            }
        }

        // Start new hunk if we have one
        if let Some(h) = hunk {
            // Save previous hunk
            if let Some(prev_hunk) = current_hunk.take() {
                file_diff.hunks.push(prev_hunk);
            }

            current_hunk = Some(DiffHunkData {
                old_start: h.old_start(),
                old_lines: h.old_lines(),
                new_start: h.new_start(),
                new_lines: h.new_lines(),
                header: String::from_utf8_lossy(h.header()).trim().to_string(),
                lines: Vec::new(),
            });
        }

        // Add line to current hunk
        if let Some(ref mut h) = current_hunk {
            let origin = line.origin();

            // Track additions and deletions
            match origin {
                '+' => file_diff.additions += 1,
                '-' => file_diff.deletions += 1,
                _ => {}
            }

            // Only include actual diff lines (not file headers)
            if matches!(origin, '+' | '-' | ' ') {
                h.lines.push(DiffLineData {
                    origin,
                    old_lineno: line.old_lineno(),
                    new_lineno: line.new_lineno(),
                    content: String::from_utf8_lossy(line.content()).to_string(),
                });
            }
        }

        true
    }).map_err(|e| format!("Failed to print diff: {}", e))?;

    // Don't forget the last hunk
    if let Some(hunk) = current_hunk {
        file_diff.hunks.push(hunk);
    }

    Ok(file_diff)
}

/// Get diff for a specific file from a commit
pub fn get_commit_file_diff(repo: &Repository, commit_id: &str, path: &str) -> Result<FileDiff, String> {
    let oid = git2::Oid::from_str(commit_id)
        .map_err(|e| format!("Invalid commit ID: {}", e))?;

    let commit = repo.find_commit(oid)
        .map_err(|e| format!("Commit not found: {}", e))?;

    let commit_tree = commit.tree()
        .map_err(|e| format!("Failed to get commit tree: {}", e))?;

    // Get parent tree (or empty tree for initial commit)
    let parent_tree = commit.parent(0)
        .ok()
        .and_then(|p| p.tree().ok());

    let mut diff_opts = DiffOptions::new();
    diff_opts.pathspec(path);
    diff_opts.context_lines(3);

    let diff = repo.diff_tree_to_tree(
        parent_tree.as_ref(),
        Some(&commit_tree),
        Some(&mut diff_opts)
    ).map_err(|e| format!("Failed to get commit diff: {}", e))?;

    // Parse the diff
    let mut file_diff = FileDiff {
        path: path.to_string(),
        old_path: None,
        hunks: Vec::new(),
        is_binary: false,
        additions: 0,
        deletions: 0,
    };

    let mut current_hunk: Option<DiffHunkData> = None;

    diff.print(DiffFormat::Patch, |delta, hunk, line| {
        if delta.flags().is_binary() {
            file_diff.is_binary = true;
            return true;
        }

        if let Some(old_file) = delta.old_file().path() {
            let old_path_str = old_file.to_string_lossy().to_string();
            if old_path_str != path {
                file_diff.old_path = Some(old_path_str);
            }
        }

        // Start new hunk if we have one
        if let Some(h) = hunk {
            if let Some(prev_hunk) = current_hunk.take() {
                file_diff.hunks.push(prev_hunk);
            }

            current_hunk = Some(DiffHunkData {
                old_start: h.old_start(),
                old_lines: h.old_lines(),
                new_start: h.new_start(),
                new_lines: h.new_lines(),
                header: String::from_utf8_lossy(h.header()).trim().to_string(),
                lines: Vec::new(),
            });
        }

        if let Some(ref mut h) = current_hunk {
            let origin = line.origin();

            match origin {
                '+' => file_diff.additions += 1,
                '-' => file_diff.deletions += 1,
                _ => {}
            }

            if matches!(origin, '+' | '-' | ' ') {
                h.lines.push(DiffLineData {
                    origin,
                    old_lineno: line.old_lineno(),
                    new_lineno: line.new_lineno(),
                    content: String::from_utf8_lossy(line.content()).to_string(),
                });
            }
        }

        true
    }).map_err(|e| format!("Failed to print diff: {}", e))?;

    if let Some(hunk) = current_hunk {
        file_diff.hunks.push(hunk);
    }

    Ok(file_diff)
}

/// Get full diff stats for repository
pub fn get_diff_stats(repo: &Repository, staged: bool) -> Result<(u32, u32, u32), String> {
    let diff = if staged {
        let head_tree = repo.head()
            .ok()
            .and_then(|h| h.peel_to_tree().ok());

        repo.diff_tree_to_index(head_tree.as_ref(), None, None)
            .map_err(|e| format!("Failed to get staged diff: {}", e))?
    } else {
        repo.diff_index_to_workdir(None, None)
            .map_err(|e| format!("Failed to get unstaged diff: {}", e))?
    };

    let stats = diff.stats()
        .map_err(|e| format!("Failed to get diff stats: {}", e))?;

    Ok((
        stats.files_changed() as u32,
        stats.insertions() as u32,
        stats.deletions() as u32,
    ))
}
