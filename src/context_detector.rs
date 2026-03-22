//! Context detector for workspace and chat analysis

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceContext {
    pub files: Vec<String>,
    pub current_file: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChatContext {
    pub query_history: Vec<String>,
}

impl ChatContext {
    pub fn add_query(&mut self, query: String) {
        self.query_history.push(query);
    }
}

/// Detects workspace and chat context for intelligent routing
#[derive(Debug, Clone, Default)]
pub struct ContextDetector {
    workspace: Option<WorkspaceContext>,
    chat: Option<ChatContext>,
}

impl ContextDetector {
    pub fn new() -> Self {
        Self {
            workspace: Some(WorkspaceContext::default()),
            chat: Some(ChatContext::default()),
        }
    }

    pub async fn get_workspace_context(&self) -> WorkspaceContext {
        self.workspace.clone().unwrap_or_default()
    }

    pub async fn get_chat_context(&self) -> ChatContext {
        self.chat.clone().unwrap_or_default()
    }

    pub async fn update_chat_context<F>(&self, _f: F)
    where
        F: Fn(&mut ChatContext),
    {
        // Stub implementation
    }
}

