//! reading the state of the git repository the editor has open

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    /// the file turned into a symlink, or a symlink turned into a file
    TypeChanged,
    Untracked,
    Ignored,
    Conflicted,
}

impl ChangeKind {
    pub fn letter(self) -> &'static str {
        match self {
            ChangeKind::Added => "A",
            ChangeKind::Modified => "M",
            ChangeKind::Deleted => "D",
            ChangeKind::Renamed => "R",
            ChangeKind::Copied => "C",
            ChangeKind::TypeChanged => "T",
            ChangeKind::Untracked => "U",
            ChangeKind::Ignored => "I",
            ChangeKind::Conflicted => "!",
        }
    }

    fn from_code(code: u8) -> Option<ChangeKind> {
        match code {
            b'A' => Some(ChangeKind::Added),
            b'M' => Some(ChangeKind::Modified),
            b'D' => Some(ChangeKind::Deleted),
            b'R' => Some(ChangeKind::Renamed),
            b'C' => Some(ChangeKind::Copied),
            b'T' => Some(ChangeKind::TypeChanged),
            b'?' => Some(ChangeKind::Untracked),
            b'!' => Some(ChangeKind::Ignored),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    pub path: String,
    pub original_path: Option<String>,
    pub index: Option<ChangeKind>,
    pub worktree: Option<ChangeKind>,
    pub conflicted: bool,
}
