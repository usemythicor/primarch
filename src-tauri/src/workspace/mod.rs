pub mod config;
pub mod storage;

pub use config::Workspace;
pub use storage::{delete_workspace, list_workspaces, load_workspace, save_workspace};
