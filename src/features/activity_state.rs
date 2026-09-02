//! Tracks which file the user is currently focused on, independent of any specific consumer
//! (Discord RPC, Wakatime, etc). This module
//! only tracks state and detects changes
//!
//! Consumers read this state after it changes (see the `ActiveFileChanged`
//! message that funnels updates into it)

use std::path::PathBuf;
use std::time::Instant;

// The file currently active in the editor, plus when the user started
// focusing on it. The `since` timestamp resets on every actual change,
// so consumers like Discord RPC can derive an "elapsed" timer from it.
#[derive(Debug, Clone)]
pub struct ActiveFileState {
    pub path: Option<PathBuf>,
    pub workspace_name: Option<String>,
    pub since: Instant,
}

impl ActiveFileState {
    pub fn empty() -> Self {
        Self {
            path: None,
            workspace_name: None,
            since: Instant::now(),
        }
    }

    /// Updates the active file/workspace if either has changed
    /// resetting the `since` timer on any actual change. Returns
    /// `true` when something changed, so callers
    /// know whether it's worth telling anyone
    pub fn update(&mut self, path: Option<PathBuf>, workspace_name: Option<String>) -> bool {
        if self.path == path && self.workspace_name == workspace_name {
            return false;
        }

        self.path = path;
        self.workspace_name = workspace_name;
        self.since = Instant::now();
        true
    }
}

impl Default for ActiveFileState {
    fn default() -> Self {
        Self::empty()
    }
}
