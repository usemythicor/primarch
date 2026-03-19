mod session;
pub mod shells;

pub use session::*;
pub use shells::{detect_shells, ShellInfo};

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Manages all terminal sessions
pub struct PtyManager {
    sessions: Arc<RwLock<HashMap<String, TerminalSession>>>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new terminal session
    pub fn create_session(
        &self,
        shell: Option<String>,
        cwd: Option<String>,
    ) -> Result<String, String> {
        let session = TerminalSession::new(shell, cwd)?;
        let id = session.id.clone();
        self.sessions.write().insert(id.clone(), session);
        Ok(id)
    }

    /// Get a session by ID
    pub fn get_session(&self, id: &str) -> Option<TerminalSession> {
        self.sessions.read().get(id).cloned()
    }

    /// Write to a session's stdin
    pub fn write_to_session(&self, id: &str, data: &[u8]) -> Result<(), String> {
        let sessions = self.sessions.read();
        if let Some(session) = sessions.get(id) {
            session.write(data)
        } else {
            Err(format!("Session {} not found", id))
        }
    }

    /// Resize a session's terminal
    pub fn resize_session(&self, id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let sessions = self.sessions.read();
        if let Some(session) = sessions.get(id) {
            session.resize(cols, rows)
        } else {
            Err(format!("Session {} not found", id))
        }
    }

    /// Kill a session
    pub fn kill_session(&self, id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.write();
        if let Some(mut session) = sessions.remove(id) {
            session.kill()
        } else {
            Err(format!("Session {} not found", id))
        }
    }

    /// Get all session IDs
    pub fn list_sessions(&self) -> Vec<String> {
        self.sessions.read().keys().cloned().collect()
    }

    /// Get the current working directory of a session
    pub fn get_session_cwd(&self, id: &str) -> Result<String, String> {
        let sessions = self.sessions.read();
        if let Some(session) = sessions.get(id) {
            session.get_cwd()
        } else {
            Err(format!("Session {} not found", id))
        }
    }

}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}
