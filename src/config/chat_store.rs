//! Persists AI chat history, scoped per workspace folder.
//!
//! This is app-managed bookkeeping rather than a user-facing setting, so
//! like `session.rs` it lives in its own plain JSON file(s) instead of the
//! hand-written Lua format used by `preferences.rs`.
//!
//! Each workspace folder gets its own file under `~/.config/pinel/chats/`,
//! so opening a different project shows that project's chat history. When
//! no folder is open, history falls back to a shared `no-workspace.json`
//! bucket.

use super::theme_manager::get_config_dir;
use crate::features::chat::ChatSession;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
struct WorkspaceChatHistory {
    workspace: Option<PathBuf>,
    sessions: Vec<ChatSession>,
}

fn chats_dir() -> PathBuf {
    get_config_dir().join("chats")
}

/// Maps a workspace folder (or `None` for "no folder open") to a stable,
/// filesystem-safe file name. The path is sanitized into the name for
/// readability, with a hash suffix to disambiguate different paths that
/// happen to sanitize to the same string.
fn workspace_file_name(workspace: Option<&Path>) -> String {
    let Some(path) = workspace else {
        return "no-workspace.json".to_string();
    };

    let raw = path.to_string_lossy();
    let mut sanitized: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else {
                '_'
            }
        })
        .collect();
    sanitized.truncate(120);

    let mut hasher = DefaultHasher::new();
    raw.hash(&mut hasher);
    let suffix = hasher.finish();

    format!("{sanitized}-{suffix:x}.json")
}

fn workspace_path(workspace: Option<&Path>) -> PathBuf {
    chats_dir().join(workspace_file_name(workspace))
}

/// Loads all chat sessions for the given workspace. Returns an empty list
/// on first run or if the file is missing/corrupted - callers should treat
/// that the same as "no history yet".
pub fn load_sessions(workspace: Option<&Path>) -> Vec<ChatSession> {
    let Ok(content) = fs::read_to_string(workspace_path(workspace)) else {
        return Vec::new();
    };

    serde_json::from_str::<WorkspaceChatHistory>(&content)
        .map(|h| h.sessions)
        .unwrap_or_default()
}

/// Persists all chat sessions for the given workspace, overwriting
/// whatever was previously saved for it.
pub fn save_sessions(workspace: Option<&Path>, sessions: &[ChatSession]) -> std::io::Result<()> {
    let dir = chats_dir();
    fs::create_dir_all(&dir)?;

    let history = WorkspaceChatHistory {
        workspace: workspace.map(Path::to_path_buf),
        sessions: sessions.to_vec(),
    };
    let json = serde_json::to_string_pretty(&history).unwrap_or_else(|_| "{}".to_string());

    fs::write(workspace_path(workspace), json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_workspace_uses_a_shared_bucket_name() {
        assert_eq!(workspace_file_name(None), "no-workspace.json");
    }

    #[test]
    fn same_workspace_path_produces_the_same_file_name() {
        let path = Path::new("/tmp/some-project");
        assert_eq!(
            workspace_file_name(Some(path)),
            workspace_file_name(Some(path))
        );
    }

    #[test]
    fn different_workspace_paths_produce_different_file_names() {
        let a = workspace_file_name(Some(Path::new("/tmp/project-a")));
        let b = workspace_file_name(Some(Path::new("/tmp/project-b")));
        assert_ne!(a, b);
    }

    #[test]
    fn workspace_history_serialize_and_deserialize_roundtrip() {
        let mut history = WorkspaceChatHistory {
            workspace: Some(PathBuf::from("/tmp/project")),
            sessions: vec![ChatSession::new("OpenAI", "gpt-4o")],
        };
        history.sessions[0].push(crate::features::chat::ChatMessage::user("hi"));

        let json = serde_json::to_string(&history).unwrap();
        let restored: WorkspaceChatHistory = serde_json::from_str(&json).unwrap();

        assert_eq!(restored, history);
    }

    #[test]
    fn deserializing_malformed_json_fails_gracefully() {
        let result: Result<WorkspaceChatHistory, _> = serde_json::from_str("{ not valid json}");
        assert!(result.is_err());
    }
}
