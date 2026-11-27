//! Asking what to do with unsaved edits before a tab or the window closes.
//!
//! The prompt is a native message dialog with Save / Don't Save / Cancel
//! buttons. Saving writes each buffer to its file, and asks for a path first
//! when the buffer has never been saved.

use std::path::{Path, PathBuf};

const SAVE_LABEL: &str = "Save";
const DISCARD_LABEL: &str = "Don't Save";
const CANCEL_LABEL: &str = "Cancel";

/// What the user picked in the unsaved-changes prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseChoice {
    /// Write the edits to disk, then close.
    Save,
    /// Close and throw the edits away.
    Discard,
    /// Keep everything open.
    Cancel,
}

/// One modified buffer to write before closing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingSave {
    /// Index of the tab the buffer belongs to when the save started.
    pub tab: usize,
    /// The tab's path when the save started; `untitled` for a new buffer.
    pub path: PathBuf,
    /// The tab's display name, used as the default file name for new buffers.
    pub name: String,
    /// The text to write.
    pub content: String,
}

/// A buffer that was written to disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedBuffer {
    /// Index of the tab the buffer belonged to when the save started.
    pub tab: usize,
    /// The tab's path when the save started.
    pub old_path: PathBuf,
    /// Where the buffer was written. Differs from `old_path` for new buffers.
    pub new_path: PathBuf,
    /// The text that was written.
    pub content: String,
}

/// Whether `path` is the placeholder path of a buffer that was never saved.
pub fn is_untitled(path: &Path) -> bool {
    path == Path::new("untitled")
}

/// The question shown in the prompt for the given unsaved buffer names.
pub fn prompt_description(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [name] => format!(
            "Do you want to save the changes you made to \"{name}\"?\n\nYour changes will be lost if you don't save them."
        ),
        _ => {
            let list = names.iter().map(|n| format!("  • {n}")).collect::<Vec<_>>().join("\n");
            format!(
                "{} files have unsaved changes:\n{list}\n\nYour changes will be lost if you don't save them.",
                names.len()
            )
        },
    }
}

/// Maps a dialog result to a choice. Platforms that ignore custom button
/// labels report Yes / No instead, so those map to Save / Don't Save.
fn choice_from_result(result: &rfd::MessageDialogResult) -> CloseChoice {
    match result {
        rfd::MessageDialogResult::Yes => CloseChoice::Save,
        rfd::MessageDialogResult::No => CloseChoice::Discard,
        rfd::MessageDialogResult::Custom(label) if label == SAVE_LABEL => CloseChoice::Save,
        rfd::MessageDialogResult::Custom(label) if label == DISCARD_LABEL => CloseChoice::Discard,
        _ => CloseChoice::Cancel,
    }
}

/// Shows the unsaved-changes prompt for `names` and waits for an answer.
pub async fn ask(names: Vec<String>) -> CloseChoice {
    let result = rfd::AsyncMessageDialog::new()
        .set_level(rfd::MessageLevel::Warning)
        .set_title("Unsaved Changes")
        .set_description(prompt_description(&names))
        .set_buttons(rfd::MessageButtons::YesNoCancelCustom(
            SAVE_LABEL.to_string(),
            DISCARD_LABEL.to_string(),
            CANCEL_LABEL.to_string(),
        ))
        .show()
        .await;
    choice_from_result(&result)
}

/// Writes every buffer in `pending`, asking for a path for new buffers.
///
/// Stops at the first buffer that can't be saved, so nothing closes when the
/// user cancels a save dialog or a write fails. The error says which file.
pub async fn save_all(pending: Vec<PendingSave>) -> Result<Vec<SavedBuffer>, String> {
    let mut saved = Vec::with_capacity(pending.len());
    for buffer in pending {
        let new_path = if is_untitled(&buffer.path) {
            rfd::AsyncFileDialog::new()
                .set_title("Save As")
                .set_file_name(&buffer.name)
                .save_file()
                .await
                .map(|handle| handle.path().to_path_buf())
                .ok_or_else(|| format!("Save of \"{}\" was cancelled", buffer.name))?
        } else {
            buffer.path.clone()
        };
        crate::fs_util::write_atomic(&new_path, &buffer.content)
            .map_err(|e| format!("Failed to save {}: {e}", new_path.display()))?;
        saved.push(SavedBuffer {
            tab: buffer.tab,
            old_path: buffer.path,
            new_path,
            content: buffer.content,
        });
    }
    Ok(saved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_untitled_placeholder() {
        assert!(is_untitled(Path::new("untitled")));
        assert!(!is_untitled(Path::new("/tmp/untitled")));
        assert!(!is_untitled(Path::new("main.rs")));
    }

    #[test]
    fn test_prompt_description_single_names_file() {
        let text = prompt_description(&["main.rs".to_string()]);
        assert!(text.contains("\"main.rs\""));
    }

    #[test]
    fn test_prompt_description_many_lists_every_file() {
        let names = vec![
            "a.rs".to_string(),
            "b.rs".to_string(),
            "untitled".to_string(),
        ];
        let text = prompt_description(&names);
        assert!(text.starts_with("3 files"));
        for name in &names {
            assert!(text.contains(name.as_str()));
        }
    }

    #[test]
    fn test_prompt_description_empty() {
        assert_eq!(prompt_description(&[]), "");
    }

    #[test]
    fn test_choice_from_result_custom_labels() {
        use rfd::MessageDialogResult as R;
        assert_eq!(
            choice_from_result(&R::Custom(SAVE_LABEL.into())),
            CloseChoice::Save
        );
        assert_eq!(
            choice_from_result(&R::Custom(DISCARD_LABEL.into())),
            CloseChoice::Discard
        );
        assert_eq!(
            choice_from_result(&R::Custom(CANCEL_LABEL.into())),
            CloseChoice::Cancel
        );
    }

    #[test]
    fn test_choice_from_result_platform_defaults() {
        use rfd::MessageDialogResult as R;
        assert_eq!(choice_from_result(&R::Yes), CloseChoice::Save);
        assert_eq!(choice_from_result(&R::No), CloseChoice::Discard);
        assert_eq!(choice_from_result(&R::Cancel), CloseChoice::Cancel);
        assert_eq!(choice_from_result(&R::Ok), CloseChoice::Cancel);
    }

    #[tokio::test]
    async fn test_save_all_writes_named_buffers() -> Result<(), Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir().join(format!("pinel-unsaved-{}", std::process::id()));
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("a.txt");
        let saved = save_all(vec![PendingSave {
            tab: 2,
            path: path.clone(),
            name: "a.txt".into(),
            content: "hello".into(),
        }])
        .await?;
        assert_eq!(std::fs::read_to_string(&path)?, "hello");
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].tab, 2);
        assert_eq!(saved[0].new_path, path);
        std::fs::remove_dir_all(&dir)?;
        Ok(())
    }

    #[tokio::test]
    async fn test_save_all_stops_on_write_error() {
        let bad = PathBuf::from("/nonexistent-pinel-dir/x/y.txt");
        let result = save_all(vec![PendingSave {
            tab: 0,
            path: bad,
            name: "y.txt".into(),
            content: "x".into(),
        }])
        .await;
        assert!(result.is_err());
    }
}
