use git2::{Repository, StatusOptions, BranchType, Signature, FetchOptions, PushOptions, RemoteCallbacks, Cred, CredentialType};
use super::{GitStatus, FileStatus, FileStatusType, BranchInfo};

/// Get the full status of a repository
pub fn get_repository_status(repo: &Repository) -> Result<GitStatus, String> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .include_ignored(false)
        .include_unmodified(false);

    let statuses = repo.statuses(Some(&mut opts))
        .map_err(|e| format!("Failed to get status: {}", e))?;

    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();
    let mut conflicted = Vec::new();

    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("").to_string();
        let status = entry.status();

        // Check for conflicts
        if status.is_conflicted() {
            conflicted.push(path.clone());
            continue;
        }

        // Check for staged changes (index)
        if status.is_index_new() {
            staged.push(FileStatus {
                path: path.clone(),
                status: FileStatusType::Added,
                old_path: None,
            });
        } else if status.is_index_modified() {
            staged.push(FileStatus {
                path: path.clone(),
                status: FileStatusType::Modified,
                old_path: None,
            });
        } else if status.is_index_deleted() {
            staged.push(FileStatus {
                path: path.clone(),
                status: FileStatusType::Deleted,
                old_path: None,
            });
        } else if status.is_index_renamed() {
            staged.push(FileStatus {
                path: path.clone(),
                status: FileStatusType::Renamed,
                old_path: entry.head_to_index()
                    .and_then(|d| d.old_file().path())
                    .map(|p| p.to_string_lossy().to_string()),
            });
        } else if status.is_index_typechange() {
            staged.push(FileStatus {
                path: path.clone(),
                status: FileStatusType::TypeChanged,
                old_path: None,
            });
        }

        // Check for unstaged changes (worktree)
        if status.is_wt_new() {
            untracked.push(path.clone());
        } else if status.is_wt_modified() {
            unstaged.push(FileStatus {
                path: path.clone(),
                status: FileStatusType::Modified,
                old_path: None,
            });
        } else if status.is_wt_deleted() {
            unstaged.push(FileStatus {
                path: path.clone(),
                status: FileStatusType::Deleted,
                old_path: None,
            });
        } else if status.is_wt_renamed() {
            unstaged.push(FileStatus {
                path: path.clone(),
                status: FileStatusType::Renamed,
                old_path: entry.index_to_workdir()
                    .and_then(|d| d.old_file().path())
                    .map(|p| p.to_string_lossy().to_string()),
            });
        } else if status.is_wt_typechange() {
            unstaged.push(FileStatus {
                path: path.clone(),
                status: FileStatusType::TypeChanged,
                old_path: None,
            });
        }
    }

    // Get branch info
    let (branch, upstream, ahead, behind) = get_branch_tracking_info(repo);

    Ok(GitStatus {
        branch,
        upstream,
        ahead,
        behind,
        staged,
        unstaged,
        untracked,
        conflicted,
    })
}

/// Get current branch name and tracking info
fn get_branch_tracking_info(repo: &Repository) -> (Option<String>, Option<String>, u32, u32) {
    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return (None, None, 0, 0),
    };

    let branch_name = head.shorthand().map(|s| s.to_string());

    if !head.is_branch() {
        return (branch_name, None, 0, 0);
    }

    // Get upstream branch
    let branch = match repo.find_branch(
        branch_name.as_ref().map(|s| s.as_str()).unwrap_or(""),
        BranchType::Local
    ) {
        Ok(b) => b,
        Err(_) => return (branch_name, None, 0, 0),
    };

    let upstream = match branch.upstream() {
        Ok(u) => u.name().ok().flatten().map(|s| s.to_string()),
        Err(_) => None,
    };

    // Calculate ahead/behind
    let (ahead, behind) = if let Some(ref _upstream_name) = upstream {
        match (branch.get().target(), branch.upstream().ok().and_then(|u| u.get().target())) {
            (Some(local), Some(remote)) => {
                repo.graph_ahead_behind(local, remote).unwrap_or((0, 0))
            }
            _ => (0, 0),
        }
    } else {
        (0, 0)
    };

    (branch_name, upstream, ahead as u32, behind as u32)
}

/// Get info about the current branch
pub fn get_current_branch(repo: &Repository) -> Result<BranchInfo, String> {
    let head = repo.head()
        .map_err(|e| format!("Failed to get HEAD: {}", e))?;

    let name = head.shorthand()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "HEAD".to_string());

    let is_head = true;

    // Get upstream info
    let (upstream, ahead, behind) = if head.is_branch() {
        let branch = repo.find_branch(&name, BranchType::Local)
            .map_err(|e| format!("Failed to find branch: {}", e))?;

        let upstream = branch.upstream()
            .ok()
            .and_then(|u| u.name().ok().flatten().map(|s| s.to_string()));

        let (a, b) = if upstream.is_some() {
            match (branch.get().target(), branch.upstream().ok().and_then(|u| u.get().target())) {
                (Some(local), Some(remote)) => {
                    repo.graph_ahead_behind(local, remote).unwrap_or((0, 0))
                }
                _ => (0, 0),
            }
        } else {
            (0, 0)
        };

        (upstream, a as u32, b as u32)
    } else {
        (None, 0, 0)
    };

    Ok(BranchInfo {
        name,
        is_head,
        upstream,
        ahead,
        behind,
    })
}

/// List all branches
pub fn list_branches(repo: &Repository) -> Result<Vec<BranchInfo>, String> {
    let mut branches = Vec::new();

    let head = repo.head().ok();
    let head_name = head.as_ref()
        .and_then(|h| h.shorthand())
        .map(|s| s.to_string());

    // Local branches
    let local_branches = repo.branches(Some(BranchType::Local))
        .map_err(|e| format!("Failed to list branches: {}", e))?;

    for branch_result in local_branches {
        let (branch, _) = branch_result
            .map_err(|e| format!("Failed to read branch: {}", e))?;

        let name = branch.name()
            .map_err(|e| format!("Failed to get branch name: {}", e))?
            .unwrap_or("")
            .to_string();

        let is_head = Some(&name) == head_name.as_ref();

        let upstream = branch.upstream()
            .ok()
            .and_then(|u| u.name().ok().flatten().map(|s| s.to_string()));

        let (ahead, behind) = if upstream.is_some() {
            match (branch.get().target(), branch.upstream().ok().and_then(|u| u.get().target())) {
                (Some(local), Some(remote)) => {
                    repo.graph_ahead_behind(local, remote).unwrap_or((0, 0))
                }
                _ => (0, 0),
            }
        } else {
            (0, 0)
        };

        branches.push(BranchInfo {
            name,
            is_head,
            upstream,
            ahead: ahead as u32,
            behind: behind as u32,
        });
    }

    Ok(branches)
}

/// Stage a single file
pub fn stage_file(repo: &Repository, path: &str) -> Result<(), String> {
    let mut index = repo.index()
        .map_err(|e| format!("Failed to get index: {}", e))?;

    // Check if file exists or was deleted
    let full_path = repo.workdir()
        .ok_or("Repository has no working directory")?
        .join(path);

    if full_path.exists() {
        index.add_path(std::path::Path::new(path))
            .map_err(|e| format!("Failed to stage file: {}", e))?;
    } else {
        // File was deleted, remove from index
        index.remove_path(std::path::Path::new(path))
            .map_err(|e| format!("Failed to stage deletion: {}", e))?;
    }

    index.write()
        .map_err(|e| format!("Failed to write index: {}", e))?;

    Ok(())
}

/// Unstage a single file
pub fn unstage_file(repo: &Repository, path: &str) -> Result<(), String> {
    let head = repo.head()
        .map_err(|e| format!("Failed to get HEAD: {}", e))?;

    let head_commit = head.peel_to_commit()
        .map_err(|e| format!("Failed to get HEAD commit: {}", e))?;

    let head_tree = head_commit.tree()
        .map_err(|e| format!("Failed to get HEAD tree: {}", e))?;

    repo.reset_default(Some(&head_commit.as_object()), &[std::path::Path::new(path)])
        .map_err(|e| format!("Failed to unstage file: {}", e))?;

    Ok(())
}

/// Stage all changes
pub fn stage_all(repo: &Repository) -> Result<(), String> {
    let mut index = repo.index()
        .map_err(|e| format!("Failed to get index: {}", e))?;

    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
        .map_err(|e| format!("Failed to stage all: {}", e))?;

    // Handle deleted files
    index.update_all(["*"].iter(), None)
        .map_err(|e| format!("Failed to update index: {}", e))?;

    index.write()
        .map_err(|e| format!("Failed to write index: {}", e))?;

    Ok(())
}

/// Create a new commit
pub fn create_commit(repo: &Repository, message: &str) -> Result<String, String> {
    let mut index = repo.index()
        .map_err(|e| format!("Failed to get index: {}", e))?;

    let tree_oid = index.write_tree()
        .map_err(|e| format!("Failed to write tree: {}", e))?;

    let tree = repo.find_tree(tree_oid)
        .map_err(|e| format!("Failed to find tree: {}", e))?;

    // Get signature from config or use default
    let sig = repo.signature()
        .or_else(|_| Signature::now("Unknown", "unknown@example.com"))
        .map_err(|e| format!("Failed to create signature: {}", e))?;

    // Get parent commit (HEAD)
    let parents = match repo.head() {
        Ok(head) => {
            let commit = head.peel_to_commit()
                .map_err(|e| format!("Failed to get HEAD commit: {}", e))?;
            vec![commit]
        }
        Err(_) => vec![], // Initial commit
    };

    let parent_refs: Vec<&git2::Commit> = parents.iter().collect();

    let oid = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        message,
        &tree,
        &parent_refs,
    ).map_err(|e| format!("Failed to create commit: {}", e))?;

    Ok(oid.to_string())
}

/// Create remote callbacks with credential handling
fn create_remote_callbacks<'a>() -> RemoteCallbacks<'a> {
    let mut callbacks = RemoteCallbacks::new();

    callbacks.credentials(|_url, username_from_url, allowed_types| {
        // Try SSH agent first
        if allowed_types.contains(CredentialType::SSH_KEY) {
            if let Some(username) = username_from_url {
                if let Ok(cred) = Cred::ssh_key_from_agent(username) {
                    return Ok(cred);
                }
            }
        }

        // Try default credentials (git credential manager)
        if allowed_types.contains(CredentialType::DEFAULT) {
            if let Ok(cred) = Cred::default() {
                return Ok(cred);
            }
        }

        // Try user's default SSH key
        if allowed_types.contains(CredentialType::SSH_KEY) {
            let username = username_from_url.unwrap_or("git");
            let home = std::env::var("USERPROFILE")
                .or_else(|_| std::env::var("HOME"))
                .unwrap_or_default();

            let ssh_key = std::path::Path::new(&home).join(".ssh").join("id_rsa");
            let ssh_pub = std::path::Path::new(&home).join(".ssh").join("id_rsa.pub");

            if ssh_key.exists() {
                return Cred::ssh_key(
                    username,
                    Some(&ssh_pub),
                    &ssh_key,
                    None,
                );
            }

            // Try ed25519 key
            let ssh_key = std::path::Path::new(&home).join(".ssh").join("id_ed25519");
            let ssh_pub = std::path::Path::new(&home).join(".ssh").join("id_ed25519.pub");

            if ssh_key.exists() {
                return Cred::ssh_key(
                    username,
                    Some(&ssh_pub),
                    &ssh_key,
                    None,
                );
            }
        }

        Err(git2::Error::from_str("No valid credentials found"))
    });

    callbacks
}

/// Fetch from remote
pub fn fetch(repo: &Repository, remote_name: &str) -> Result<(), String> {
    let mut remote = repo.find_remote(remote_name)
        .map_err(|e| format!("Remote '{}' not found: {}", remote_name, e))?;

    let callbacks = create_remote_callbacks();
    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    // Fetch all branches
    remote.fetch(&[] as &[&str], Some(&mut fetch_opts), None)
        .map_err(|e| format!("Fetch failed: {}", e))?;

    Ok(())
}

/// Pull from remote (fetch + merge)
pub fn pull(repo: &Repository, remote_name: &str) -> Result<String, String> {
    // First, fetch
    fetch(repo, remote_name)?;

    // Get current branch
    let head = repo.head()
        .map_err(|e| format!("Failed to get HEAD: {}", e))?;

    if !head.is_branch() {
        return Err("HEAD is not on a branch".to_string());
    }

    let branch_name = head.shorthand()
        .ok_or("Invalid branch name")?;

    // Get upstream reference
    let fetch_head = repo.find_reference("FETCH_HEAD")
        .map_err(|e| format!("No FETCH_HEAD found: {}", e))?;

    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)
        .map_err(|e| format!("Failed to get fetch commit: {}", e))?;

    // Perform merge analysis
    let (analysis, _) = repo.merge_analysis(&[&fetch_commit])
        .map_err(|e| format!("Merge analysis failed: {}", e))?;

    if analysis.is_up_to_date() {
        return Ok("Already up to date".to_string());
    }

    if analysis.is_fast_forward() {
        // Fast-forward merge
        let refname = format!("refs/heads/{}", branch_name);
        let mut reference = repo.find_reference(&refname)
            .map_err(|e| format!("Failed to find branch reference: {}", e))?;

        reference.set_target(fetch_commit.id(), "Fast-forward")
            .map_err(|e| format!("Failed to fast-forward: {}", e))?;

        repo.set_head(&refname)
            .map_err(|e| format!("Failed to set HEAD: {}", e))?;

        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .map_err(|e| format!("Failed to checkout: {}", e))?;

        return Ok("Fast-forward merge complete".to_string());
    }

    if analysis.is_normal() {
        // Normal merge required
        return Err("Merge required - please resolve manually".to_string());
    }

    Err("Cannot pull - unknown state".to_string())
}

/// Push to remote
pub fn push(repo: &Repository, remote_name: &str) -> Result<(), String> {
    let head = repo.head()
        .map_err(|e| format!("Failed to get HEAD: {}", e))?;

    if !head.is_branch() {
        return Err("HEAD is not on a branch".to_string());
    }

    let branch_name = head.shorthand()
        .ok_or("Invalid branch name")?;

    let mut remote = repo.find_remote(remote_name)
        .map_err(|e| format!("Remote '{}' not found: {}", remote_name, e))?;

    let callbacks = create_remote_callbacks();
    let mut push_opts = PushOptions::new();
    push_opts.remote_callbacks(callbacks);

    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

    remote.push(&[&refspec], Some(&mut push_opts))
        .map_err(|e| format!("Push failed: {}", e))?;

    Ok(())
}

/// Get list of remotes
pub fn list_remotes(repo: &Repository) -> Result<Vec<String>, String> {
    let remotes = repo.remotes()
        .map_err(|e| format!("Failed to list remotes: {}", e))?;

    Ok(remotes.iter()
        .filter_map(|r| r.map(|s| s.to_string()))
        .collect())
}
