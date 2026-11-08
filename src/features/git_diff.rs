//! turning `git show` patch output into files, hunks and numbered lines

use std::path::{Path, PathBuf};

/// Lines kept per file. A generated file can change millions of lines, and
/// every kept line becomes a widget.
pub const MAX_LINES_PER_FILE: usize = 2000;

/// What a diff line did to the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Context,
    Added,
    Removed,
}

/// One line of a hunk, with its line number on each side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: LineKind,
    /// The line number before the commit; `None` for an added line.
    pub old_line: Option<usize>,
    /// The line number after the commit; `None` for a removed line.
    pub new_line: Option<usize>,
    pub text: String,
}

/// A run of changed lines with its `@@ -a,b +c,d @@` header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

/// How a commit touched one file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

/// Everything a commit changed in one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDiff {
    pub old_path: String,
    pub new_path: String,
    pub status: FileStatus,
    pub binary: bool,
    pub hunks: Vec<Hunk>,
    /// Lines added and removed, counted even when the lines are not kept.
    pub added: usize,
    pub removed: usize,
    /// Lines past [`MAX_LINES_PER_FILE`] that were counted but dropped.
    pub hidden_lines: usize,
}

impl FileDiff {
    /// The path to show: where the file was if it is gone, else where it is.
    pub fn display_path(&self) -> &str {
        match self.status {
            FileStatus::Deleted => &self.old_path,
            _ => &self.new_path,
        }
    }
}

/// Reads the starting line numbers from `-1,3 +1,4 @@ ...`.
fn hunk_starts(header: &str) -> (usize, usize) {
    let mut parts = header.split_whitespace();
    let start = |part: Option<&str>, sign: char| {
        part.and_then(|p| p.strip_prefix(sign))
            .and_then(|p| p.split(',').next())
            .and_then(|n| n.parse().ok())
            .unwrap_or(1)
    };
    (start(parts.next(), '-'), start(parts.next(), '+'))
}

/// Parses the output of the `git show --patch` run by [`load_commit_diff`].
///
/// Paths come from the `diff --git a/x b/y` line, so a path that itself
/// contains ` b/` is split in the wrong place. Renames are corrected by
/// their `rename from`/`rename to` lines.
pub fn parse_diff(output: &str) -> Vec<FileDiff> {
    let mut files: Vec<FileDiff> = Vec::new();
    let (mut old_no, mut new_no) = (0, 0);
    let mut kept = 0;

    for line in output.lines() {
        if let Some(rest) = line.strip_prefix("diff --git a/") {
            let (old, new) = rest.split_once(" b/").unwrap_or((rest, rest));
            files.push(FileDiff {
                old_path: old.to_string(),
                new_path: new.to_string(),
                status: FileStatus::Modified,
                binary: false,
                hunks: Vec::new(),
                added: 0,
                removed: 0,
                hidden_lines: 0,
            });
            kept = 0;
            continue;
        }
        let Some(file) = files.last_mut() else {
            continue;
        };

        if let Some(rest) = line.strip_prefix("@@ ") {
            (old_no, new_no) = hunk_starts(rest);
            file.hunks.push(Hunk {
                header: line.to_string(),
                lines: Vec::new(),
            });
            continue;
        }

        let Some(hunk) = file.hunks.last_mut() else {
            // still in the header block: modes, renames and "Binary files"
            if line.starts_with("new file mode") {
                file.status = FileStatus::Added;
            } else if line.starts_with("deleted file mode") {
                file.status = FileStatus::Deleted;
            } else if let Some(from) = line.strip_prefix("rename from ") {
                file.status = FileStatus::Renamed;
                file.old_path = from.to_string();
            } else if let Some(to) = line.strip_prefix("rename to ") {
                file.new_path = to.to_string();
            } else if line.starts_with("Binary files ") || line.starts_with("GIT binary patch") {
                file.binary = true;
            }
            continue;
        };

        let (kind, text) = match line.as_bytes().first() {
            Some(b'+') => (LineKind::Added, &line[1..]),
            Some(b'-') => (LineKind::Removed, &line[1..]),
            Some(b' ') => (LineKind::Context, &line[1..]),
            // "\ No newline at end of file" belongs to no line
            _ => continue,
        };
        let (old_line, new_line) = match kind {
            LineKind::Context => {
                let at = (Some(old_no), Some(new_no));
                old_no += 1;
                new_no += 1;
                at
            },
            LineKind::Added => {
                file.added += 1;
                let at = (None, Some(new_no));
                new_no += 1;
                at
            },
            LineKind::Removed => {
                file.removed += 1;
                let at = (Some(old_no), None);
                old_no += 1;
                at
            },
        };

        if kept < MAX_LINES_PER_FILE {
            kept += 1;
            hunk.lines.push(DiffLine {
                kind,
                old_line,
                new_line,
                text: text.to_string(),
            });
        } else {
            file.hidden_lines += 1;
        }
    }

    files
}

/// Reads the diff of commit `hash` from the repository at `root`.
///
/// Merges are shown against their first parent, since the combined diff
/// git prints by default has no single before and after.
pub fn load_commit_diff(root: Option<&Path>, hash: &str) -> Vec<FileDiff> {
    let mut cmd = std::process::Command::new("git");
    cmd.args([
        "-c",
        "core.quotepath=false",
        "show",
        "--format=",
        "--patch",
        "--no-color",
        "--no-ext-diff",
        "-M",
        "-m",
        "--first-parent",
    ])
    .arg(hash);
    if let Some(dir) = root {
        cmd.current_dir(dir);
    }

    match cmd.output() {
        Ok(out) if out.status.success() => parse_diff(&String::from_utf8_lossy(&out.stdout)),
        _ => Vec::new(),
    }
}

/// runs [`load_commit_diff`] off the UI thread
pub async fn load_commit_diff_async(root: Option<PathBuf>, hash: String) -> Vec<FileDiff> {
    tokio::task::spawn_blocking(move || load_commit_diff(root.as_deref(), &hash))
        .await
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r"diff --git a/src/a.rs b/src/a.rs
index 111..222 100644
--- a/src/a.rs
+++ b/src/a.rs
@@ -1,3 +1,4 @@ fn main() {
 keep
-old
+new
+extra
 tail
@@ -20 +21 @@
-x
+y
diff --git a/new.txt b/new.txt
new file mode 100644
index 0000000..333
--- /dev/null
+++ b/new.txt
@@ -0,0 +1 @@
+hello
\ No newline at end of file
diff --git a/gone.txt b/gone.txt
deleted file mode 100644
--- a/gone.txt
+++ /dev/null
@@ -1 +0,0 @@
-bye
diff --git a/img.png b/img.png
index 444..555 100644
Binary files a/img.png and b/img.png differ
diff --git a/old.rs b/moved.rs
similarity index 100%
rename from old.rs
rename to moved.rs
";

    #[test]
    fn test_parse_diff_empty_output() {
        assert!(parse_diff("").is_empty());
    }

    #[test]
    fn test_parse_diff_numbers_lines_on_both_sides() {
        let files = parse_diff(SAMPLE);
        assert_eq!(files.len(), 5);

        let a = &files[0];
        assert_eq!((a.added, a.removed), (3, 2));
        assert_eq!(a.hunks.len(), 2);
        let lines = &a.hunks[0].lines;
        assert_eq!((lines[0].old_line, lines[0].new_line), (Some(1), Some(1)));
        assert_eq!(
            (lines[1].kind, lines[1].old_line, lines[1].new_line),
            (LineKind::Removed, Some(2), None)
        );
        assert_eq!(
            (lines[2].kind, lines[2].old_line, lines[2].new_line),
            (LineKind::Added, None, Some(2))
        );
        assert_eq!(lines[3].new_line, Some(3));
        assert_eq!((lines[4].old_line, lines[4].new_line), (Some(3), Some(4)));
        // a hunk header without a count still restarts the numbering
        assert_eq!(a.hunks[1].lines[0].old_line, Some(20));
        assert_eq!(a.hunks[1].lines[1].new_line, Some(21));
    }

    #[test]
    fn test_parse_diff_added_deleted_and_no_newline_marker() {
        let files = parse_diff(SAMPLE);
        assert_eq!(files[1].status, FileStatus::Added);
        assert_eq!(files[1].hunks[0].lines.len(), 1);
        assert_eq!(files[2].status, FileStatus::Deleted);
        assert_eq!(files[2].display_path(), "gone.txt");
    }

    #[test]
    fn test_parse_diff_binary_file_has_no_hunks() {
        let files = parse_diff(SAMPLE);
        assert!(files[3].binary);
        assert!(files[3].hunks.is_empty());
    }

    #[test]
    fn test_parse_diff_rename_keeps_both_paths() {
        let files = parse_diff(SAMPLE);
        assert_eq!(files[4].status, FileStatus::Renamed);
        assert_eq!(files[4].old_path, "old.rs");
        assert_eq!(files[4].new_path, "moved.rs");
        assert!(files[4].hunks.is_empty());
    }

    #[test]
    fn test_parse_diff_removed_line_starting_with_dashes_is_not_a_header() {
        let text = "diff --git a/x b/x\n@@ -1 +1 @@\n--- a comment\n+++ b comment\n";
        let files = parse_diff(text);
        assert_eq!(files.len(), 1);
        assert_eq!((files[0].added, files[0].removed), (1, 1));
    }

    #[test]
    fn test_parse_diff_caps_kept_lines_but_counts_all() {
        let mut text = String::from("diff --git a/big b/big\n@@ -0,0 +1,9999 @@\n");
        for _ in 0..MAX_LINES_PER_FILE + 5 {
            text.push_str("+x\n");
        }
        let files = parse_diff(&text);
        assert_eq!(files[0].hunks[0].lines.len(), MAX_LINES_PER_FILE);
        assert_eq!(files[0].hidden_lines, 5);
        assert_eq!(files[0].added, MAX_LINES_PER_FILE + 5);
    }

    #[test]
    fn test_hunk_starts_defaults_to_one_when_unreadable() {
        assert_eq!(hunk_starts("-12,3 +14,5 @@ ctx"), (12, 14));
        assert_eq!(hunk_starts("garbage"), (1, 1));
    }
}
