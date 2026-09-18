//! Persists the workspace exactly as the user left it - open folder,
//! open tabs, which one was active, and each tab's cursor position
//!
//! This is app-managed bookkeeping rather than a user-facing setting
//! so unlike `preferences.rs` (which uses a hand-written Lua-ish format
//! meant to be human/Lua-editable) this lives in its own plain JSON file.

use super::theme_manager::get_config_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SessionState {
    pub folder: Option<PathBuf>,
    pub open_tabs: Vec<PathBuf>,
    pub active_tab_index: Option<PathBuf>,
    pub cursor_position: HashMap<PathBuf, (usize, usize)>,
}

fn session_path() -> PathBuf {
    get_config_dir().join("session.json")
}

// loads the last saved session, if any exists and is readable/parsable
// Returns `None` on first run, a corrupted file, or any I/O error - all of
// which should be treated the same way by callers: just start fresh.
pub fn load_session() -> Option<SessionState> {
    let content = fs::read_to_string(session_path()).ok()?;
    serde_json::from_str(&content).ok()
}

// writes the given session state to disk, creating the config directory
// if it doesnt exist yet.
pub fn save_session(state: &SessionState) -> std::io::Result<()> {
    let dir = get_config_dir();
    fs::create_dir_all(&dir)?;

    let json = serde_json::to_string_pretty(state).unwrap_or_else(|_| "{}".to_string());

    fs::write(session_path(), json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_session_state_is_empty {
        let state = SessionState::default();
        assert!(state.folder.is_none());
        assert!(state.open_tabs.is_empty());
        assert!(state.active_tab_index.is_none());
        assert!(state.cursor_position.is_empty());
    }

    #[test]
    fn session_state_serialize_and_deserialize_roundtrip() {
        let mut state = SessionState {
            folder: Some(PathBuf::from("/tmp/project")),
            open_tabs: vec![PathBuf::from("/tmp/project/a.rs"), PathBuf::from("/tmp/project/b.rs")],
            active_tab_index: Some(PathBuf::from("/tmp/project/a.rs")),
            cursor_position: HashMap::new(),
        };
        state
            .cursor_position
            .insert(PathBuf::from("/tmp/project/a.rs"), (12, 4));

        let json = serde_json::to_string(&state).unwrap();
        let restored: SessionState = serde_json::from_str(&json).unwrap();

        assert_eq!(restored, state);
    }

    #[test]
    fn deserializing_malformed_json_fails_gracefully() {
        let result: Result<SessionState, _> = serde_json::from_str("{ not valid json}");
        assert!(result.is_err());
    }
}
