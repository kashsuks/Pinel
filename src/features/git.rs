//! reading the state of the git repository the editor has open

use std::path::{Path, PathBuf};

/// what happened to a file, as reported by `git status`
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
    /// the single letter shown next to the file name in the git panel
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

/// one changed path in the working tree
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    pub path: String,
    pub original_path: Option<String>,
    pub index: Option<ChangeKind>,
    pub worktree: Option<ChangeKind>,
    pub conflicted: bool,
}

// dead_code: is_staged/is_unstaged are only exercised by tests until the
// staged/unstaged grouping lands in the git panel
#[allow(dead_code)]
impl FileChange {
    /// the kind to show when there is only room for one letter
    ///
    /// a conflict outranks everything
    pub fn primary_kind(&self) -> ChangeKind {
        if self.conflicted {
            return ChangeKind::Conflicted;
        }
        self.worktree.or(self.index).unwrap_or(ChangeKind::Modified)
    }

    /// whether any part of this change is staged for the next commit
    pub fn is_staged(&self) -> bool {
        !self.conflicted && self.index.is_some()
    }

    /// whether any part of this change is still only in the working tree
    pub fn is_unstaged(&self) -> bool {
        self.conflicted || self.worktree.is_some()
    }

    /// the part of [`path`](Self::path) to show when space is tight
    ///
    /// Untracked directories keep their trailing /
    pub fn display_name(&self) -> &str {
        let trimmed = self.path.trim_end_matches('/');
        let name = trimmed.rsplit('/').next().unwrap_or(trimmed);
        if self.path.ends_with('/') {
            &self.path[self.path.len() - name.len() - 1..]
        } else {
            name
        }
    }
}

/// Parses the output of `git status --porcelain=v1 -z`.
///
/// `-z` is what makes this reliable. Without it git wraps any path holding
/// a space or a non-ASCII byte in quotes and writes a rename as
/// `old -> new` on one line, so a path cannot be told from its own
/// punctuation. With `-z` every path is raw and NUL-terminated, and a
/// rename or copy simply writes the original path as the next field.
///
/// Records that are too short to hold a status and a path are skipped
/// rather than reported, since a partial read is not worth an error.
pub fn parse_status(output: &[u8]) -> Vec<FileChange> {
    let mut fields = output.split(|byte| *byte == 0);
    let mut changes = Vec::new();

    while let Some(record) = fields.next() {
        // XY path is three bytes before the path even starts
        if record.len() < 4 {
            continue;
        }

        let (index_code, worktree_code) = (record[0], record[1]);
        let path = String::from_utf8_lossy(&record[3..]).into_owned();

        let original_path =
            if matches!(index_code, b'R' | b'C') || matches!(worktree_code, b'R' | b'C') {
                fields.next().map(|f| String::from_utf8_lossy(f).into_owned())
            } else {
                None
            };

        let conflicted = index_code == b'U'
            || worktree_code == b'U'
            || matches!((index_code, worktree_code), (b'D', b'D') | (b'A', b'A'));

        // '??' and '!!' fill both columns, but an untracked file is not
        // staged for anything, so it only counts as a working tree change
        let (index, worktree) = match index_code {
            b'?' | b'!' => (None, ChangeKind::from_code(index_code)),
            _ => (
                ChangeKind::from_code(index_code),
                ChangeKind::from_code(worktree_code),
            ),
        };

        changes.push(FileChange {
            path,
            original_path,
            index,
            worktree,
            conflicted,
        });
    }

    changes
}

/// runs `git status` in `root` and returns the changed files
///
/// returns an empty list when `root` is not a repository, git is missing,
/// or the command fails
pub fn load_status(root: Option<&Path>) -> Vec<FileChange> {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("status").arg("--porcelain=v1").arg("-z");
    if let Some(dir) = root {
        cmd.current_dir(dir);
    }

    match cmd.output() {
        Ok(out) if out.status.success() => parse_status(&out.stdout),
        _ => Vec::new(),
    }
}

/// runs [`load_status`] off the UI thread
pub async fn load_status_async(root: Option<PathBuf>) -> Vec<FileChange> {
    tokio::task::spawn_blocking(move || load_status(root.as_deref()))
        .await
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status_empty_output() {
        assert!(parse_status(b"").is_empty());
    }

    #[test]
    fn test_parse_status_staged_and_unstaged() {
        let changes = parse_status(b"MM a.rs\0 M b.rs\0A  c.rs\0");
        assert_eq!(changes.len(), 3);
        assert_eq!(changes[0].index, Some(ChangeKind::Modified));
        assert_eq!(changes[0].worktree, Some(ChangeKind::Modified));
        assert!(changes[1].is_unstaged() && !changes[1].is_staged());
        assert!(changes[2].is_staged() && !changes[2].is_unstaged());
    }

    #[test]
    fn test_parse_status_untracked_is_not_staged() {
        let changes = parse_status(b"?? new.txt\0");
        assert!(!changes[0].is_staged());
        assert_eq!(changes[0].primary_kind(), ChangeKind::Untracked);
    }

    #[test]
    fn test_parse_status_rename_keeps_both_paths() {
        let changes = parse_status(b"R  new name.txt\0old name.txt\0 M after.rs\0");
        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0].path, "new name.txt");
        assert_eq!(changes[0].original_path.as_deref(), Some("old name.txt"));
        assert_eq!(changes[1].path, "after.rs");
    }

    #[test]
    fn test_parse_status_conflicts() {
        for record in [&b"UU f\0"[..], b"AA f\0", b"DD f\0", b"AU f\0"] {
            let changes = parse_status(record);
            assert!(changes[0].conflicted);
            assert_eq!(changes[0].primary_kind(), ChangeKind::Conflicted);
        }
    }

    #[test]
    fn test_parse_status_short_records_skipped() {
        assert!(parse_status(b"M\0 M\0").is_empty());
    }

    #[test]
    fn test_display_name_last_segment() {
        let change = |path: &str| FileChange {
            path: path.to_string(),
            original_path: None,
            index: None,
            worktree: Some(ChangeKind::Modified),
            conflicted: false,
        };
        assert_eq!(change("src/app.rs").display_name(), "app.rs");
        assert_eq!(change("top.rs").display_name(), "top.rs");
        assert_eq!(change("a/sub/").display_name(), "sub/");
    }
}
