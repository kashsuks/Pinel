//! Remembers the folders the user has opened, most recent first, so the
//! empty-editor screen can offer them again.
//!
//! Like `session.rs`, this is app-managed bookkeeping, so it lives in its own
//! plain JSON file rather than in the Lua preferences.

use super::theme_manager::get_config_dir;
use std::fs;
use std::path::{Path, PathBuf};

/// How many folders the list keeps; older ones fall off the end.
pub const MAX_RECENT_FOLDERS: usize = 10;

fn recent_folders_path() -> PathBuf {
    get_config_dir().join("recent_folders.json")
}

/// Loads the saved list of recent folders, most recent first.
///
/// A missing, unreadable or malformed file gives an empty list, since a lost
/// history is not worth bothering the user about.
pub fn load() -> Vec<PathBuf> {
    fs::read_to_string(recent_folders_path())
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

/// Writes the list of recent folders, creating the config directory if needed.
pub fn save(folders: &[PathBuf]) -> std::io::Result<()> {
    fs::create_dir_all(get_config_dir())?;
    let json = serde_json::to_string_pretty(folders).map_err(std::io::Error::other)?;
    fs::write(recent_folders_path(), json)
}

/// Moves `folder` to the front of `folders`, dropping any earlier entry for it
/// and trimming the list to [`MAX_RECENT_FOLDERS`].
pub fn push(folders: &mut Vec<PathBuf>, folder: &Path) {
    folders.retain(|existing| existing != folder);
    folders.insert(0, folder.to_path_buf());
    folders.truncate(MAX_RECENT_FOLDERS);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_adds_to_the_front() {
        let mut folders = vec![PathBuf::from("/a")];
        push(&mut folders, Path::new("/b"));
        assert_eq!(folders, vec![PathBuf::from("/b"), PathBuf::from("/a")]);
    }

    #[test]
    fn test_push_moves_an_existing_folder_to_the_front() {
        let mut folders = vec![
            PathBuf::from("/a"),
            PathBuf::from("/b"),
            PathBuf::from("/c"),
        ];
        push(&mut folders, Path::new("/c"));
        assert_eq!(
            folders,
            vec![
                PathBuf::from("/c"),
                PathBuf::from("/a"),
                PathBuf::from("/b")
            ]
        );
    }

    #[test]
    fn test_push_caps_the_list_length() {
        let mut folders: Vec<PathBuf> =
            (0..MAX_RECENT_FOLDERS).map(|i| PathBuf::from(format!("/{i}"))).collect();
        push(&mut folders, Path::new("/new"));
        assert_eq!(folders.len(), MAX_RECENT_FOLDERS);
        assert_eq!(folders[0], PathBuf::from("/new"));
        assert!(!folders.contains(&PathBuf::from(format!("/{}", MAX_RECENT_FOLDERS - 1))));
    }

    #[test]
    fn test_load_format_roundtrips_through_json() -> Result<(), serde_json::Error> {
        let folders = vec![PathBuf::from("/tmp/project"), PathBuf::from("/tmp/other")];
        let json = serde_json::to_string(&folders)?;
        let restored: Vec<PathBuf> = serde_json::from_str(&json)?;
        assert_eq!(restored, folders);
        Ok(())
    }
}
