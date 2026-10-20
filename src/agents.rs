//! Coding-agent session management, confidence tracking, and metadata.

use std::time::Instant;

use crate::worktree::WorktreeInfo;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Confidence {
    Confirmed,
    Inferred,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AgentState {
    Running,
    WaitingForInput,
    Completed,
    Failed(String),
    Idle,
}

#[derive(Clone, Debug)]
pub struct AgentSession {
    #[allow(dead_code)]
    pub id: String,
    pub name: String,
    pub state: AgentState,
    pub confidence: Confidence,
    pub start_time: Instant,
    pub target_file_or_task: String,
    pub worktree_branch: String,
    pub worktree: Option<WorktreeInfo>,
    pub terminal_idx: Option<usize>,
}

impl AgentSession {
    pub fn new(id: &str, name: &str, task: &str, branch: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            state: AgentState::Running,
            confidence: Confidence::Confirmed,
            start_time: Instant::now(),
            target_file_or_task: task.to_string(),
            worktree_branch: branch.to_string(),
            worktree: None,
            terminal_idx: None,
        }
    }

    pub fn formatted_runtime(&self) -> String {
        let secs = self.start_time.elapsed().as_secs();
        let mins = secs / 60;
        if mins > 0 {
            format!("{}m", mins)
        } else {
            format!("{}s", secs)
        }
    }
}
