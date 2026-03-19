use serde::{Deserialize, Serialize};

/// Represents a workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub layout: LayoutNode,
}

/// Represents a node in the layout tree
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LayoutNode {
    Split {
        id: String,
        direction: SplitDirection,
        ratio: f64,
        children: Box<(LayoutNode, LayoutNode)>,
    },
    Terminal {
        id: String,
        shell: Option<String>,
        cwd: Option<String>,
        startup_command: Option<String>,
        title: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

impl Workspace {
    /// Create a new workspace with generated ID and timestamps
    #[allow(dead_code)] // Available for server-side workspace creation
    pub fn new(name: String, layout: LayoutNode) -> Self {
        let now = chrono_lite::Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            created_at: now.clone(),
            updated_at: now,
            layout,
        }
    }
}

impl Default for LayoutNode {
    fn default() -> Self {
        LayoutNode::Terminal {
            id: uuid::Uuid::new_v4().to_string(),
            shell: None,
            cwd: None,
            startup_command: None,
            title: None,
        }
    }
}

/// Lightweight datetime for timestamps
#[allow(dead_code)]
mod chrono_lite {
    pub struct Utc;

    impl Utc {
        pub fn now() -> String {
            // Simple ISO 8601 timestamp
            let duration = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap();
            let secs = duration.as_secs();
            format!("{}Z", secs)
        }
    }
}
