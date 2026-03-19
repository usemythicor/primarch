use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// Manages file system watchers for git repositories
pub struct WatcherManager {
    watchers: Arc<RwLock<HashMap<String, WatcherHandle>>>,
}

struct WatcherHandle {
    // Kept alive to maintain the watcher subscription
    #[allow(dead_code)]
    watcher: RecommendedWatcher,
    #[allow(dead_code)]
    path: PathBuf,
}

impl WatcherManager {
    pub fn new() -> Self {
        Self {
            watchers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start watching a repository for file changes
    pub fn start_watching(
        &self,
        app: AppHandle,
        repo_id: String,
        repo_path: PathBuf,
    ) -> Result<(), String> {
        // Stop any existing watcher for this repo
        self.stop_watching(&repo_id);

        let (tx, rx) = channel();
        let repo_id_clone = repo_id.clone();

        // Create watcher with debouncing
        let config = Config::default().with_poll_interval(Duration::from_secs(1));

        let mut watcher = RecommendedWatcher::new(tx, config)
            .map_err(|e| format!("Failed to create watcher: {}", e))?;

        // Watch the repository directory
        watcher
            .watch(&repo_path, RecursiveMode::Recursive)
            .map_err(|e| format!("Failed to watch directory: {}", e))?;

        // Store the watcher
        self.watchers.write().insert(
            repo_id.clone(),
            WatcherHandle {
                watcher,
                path: repo_path.clone(),
            },
        );

        // Spawn thread to process events
        thread::spawn(move || {
            // Debounce timer
            let debounce_duration = Duration::from_millis(500);
            let mut last_event_time = std::time::Instant::now();
            let mut pending_emit = false;

            loop {
                // Check for events with timeout
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(Ok(event)) => {
                        // Ignore .git internal changes (too noisy)
                        let dominated_by_git = event
                            .paths
                            .iter()
                            .all(|p| p.components().any(|c| c.as_os_str() == ".git"));

                        if !dominated_by_git {
                            last_event_time = std::time::Instant::now();
                            pending_emit = true;
                        }
                    }
                    Ok(Err(_)) => {
                        // Watcher error, continue
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // Check if we should emit
                        if pending_emit && last_event_time.elapsed() >= debounce_duration {
                            let _ = app.emit(&format!("git-files-changed-{}", repo_id_clone), ());
                            pending_emit = false;
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        // Channel closed, watcher was stopped
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop watching a repository
    pub fn stop_watching(&self, repo_id: &str) {
        self.watchers.write().remove(repo_id);
    }
}

impl Default for WatcherManager {
    fn default() -> Self {
        Self::new()
    }
}
