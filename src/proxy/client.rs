use std::collections::HashMap;
use crate::config::McpServerConfig;
use crate::proxy::types::{GroupInfo, FailedGroupInfo, ToolInfo};

pub enum GroupState {
    Connected {
        name: String,
        description: String,
        tools: Vec<ToolInfo>,
    },
    Failed {
        name: String,
        description: String,
        error: String,
    },
}

pub struct ModularMcpClient {
    groups: HashMap<String, GroupState>,
}

impl ModularMcpClient {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    pub fn list_groups(&self) -> Vec<GroupInfo> {
        self.groups.values()
            .filter_map(|state| match state {
                GroupState::Connected { name, description, .. } => {
                    Some(GroupInfo {
                        name: name.clone(),
                        description: description.clone(),
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub fn list_failed_groups(&self) -> Vec<FailedGroupInfo> {
        self.groups.values()
            .filter_map(|state| match state {
                GroupState::Failed { name, description, error } => {
                    Some(FailedGroupInfo {
                        name: name.clone(),
                        description: description.clone(),
                        error: error.clone(),
                    })
                }
                _ => None,
            })
            .collect()
    }
}
