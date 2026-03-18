pub mod config;
pub mod storage;

pub use config::{LayoutNode, SplitDirection, Workspace};
pub use storage::{
    delete_workspace, list_workspaces, load_workspace, save_workspace, workspace_exists,
};
