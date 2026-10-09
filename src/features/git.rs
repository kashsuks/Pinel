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

/// How many commits the panel loads at once.
///
/// The graph is only as tall as the sidebar, so the limit exists to keep
/// the walk short on a repository with a long history rather than to fill
/// the view.
pub const HISTORY_LIMIT: usize = 500;

/// What a name attached to a commit refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefKind {
    LocalBranch,
    RemoteBranch,
    Tag,
    /// `HEAD` itself, with no branch under it. A detached checkout.
    Head,
}

/// A branch, tag or `HEAD` pointing at a commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitRef {
    /// The name without its `refs/` prefix, such as `master`,
    /// `origin/master` or `v1.0`.
    pub name: String,
    pub kind: RefKind,
    /// Whether this is the ref the working tree is currently on.
    pub is_head: bool,
}

/// One commit in the history, with what it points at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub hash: String,
    /// Parent hashes, oldest branch first. Empty for a root commit, two or
    /// more for a merge.
    pub parents: Vec<String>,
    /// The first line of the commit message.
    pub subject: String,
    /// Branches and tags that point here. Usually empty.
    pub refs: Vec<GitRef>,
    pub author: String,
    /// Author date, as seconds since the Unix epoch.
    pub timestamp: i64,
}

// Nothing draws the history yet, so these are only exercised by tests.
#[allow(dead_code)]
impl Commit {
    /// The abbreviated hash shown in the panel.
    pub fn short_hash(&self) -> &str {
        &self.hash[..self.hash.len().min(7)]
    }

    /// Whether this commit joins two or more lines of history.
    pub fn is_merge(&self) -> bool {
        self.parents.len() > 1
    }
}

/// Separates the fields within one commit record.
const FIELD: u8 = 0x1f;
/// Separates one commit record from the next.
const RECORD: u8 = 0x1e;

/// Parses the `%D` decoration list of one commit.
///
/// Reads the output of `--decorate=full`, where every name arrives fully
/// qualified. The short form cannot be read reliably, because a local
/// branch called `feature/x` and a remote branch called `origin/x` both
/// look like a name with a slash in it.
fn parse_refs(decoration: &str) -> Vec<GitRef> {
    let mut refs = Vec::new();

    for entry in decoration.split(", ") {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }

        // `HEAD -> refs/heads/main` means HEAD is on that branch.
        let (is_head, name) = match entry.strip_prefix("HEAD -> ") {
            Some(rest) => (true, rest),
            None => (false, entry),
        };

        if name == "HEAD" {
            refs.push(GitRef {
                name: "HEAD".to_string(),
                kind: RefKind::Head,
                is_head: true,
            });
            continue;
        }

        let (kind, name) = if let Some(tag) = name.strip_prefix("tag: ") {
            (RefKind::Tag, tag.strip_prefix("refs/tags/").unwrap_or(tag))
        } else if let Some(remote) = name.strip_prefix("refs/remotes/") {
            (RefKind::RemoteBranch, remote)
        } else if let Some(local) = name.strip_prefix("refs/heads/") {
            (RefKind::LocalBranch, local)
        } else {
            // Anything else is a ref namespace we do not model, such as
            // refs/stash. Keep the name so it is at least visible.
            (RefKind::LocalBranch, name)
        };

        refs.push(GitRef {
            name: name.to_string(),
            kind,
            is_head,
        });
    }

    refs
}

/// Parses the output of the `git log` run by [`load_history`].
///
/// Records are separated by `0x1e` and fields within them by `0x1f`,
/// control bytes that cannot appear in a branch name or a commit subject.
/// A record missing fields is skipped rather than guessed at.
pub fn parse_log(output: &[u8]) -> Vec<Commit> {
    let mut commits = Vec::new();

    for record in output.split(|byte| *byte == RECORD) {
        // git puts a newline after each record, which lands at the front
        // of the next one.
        let record: &[u8] = match record.iter().position(|b| !b" \n\r".contains(b)) {
            Some(start) => &record[start..],
            None => continue,
        };

        let fields: Vec<&[u8]> = record.split(|byte| *byte == FIELD).collect();
        let [hash, parents, subject, decoration, author, timestamp] = fields[..] else {
            continue;
        };

        let text = |bytes: &[u8]| String::from_utf8_lossy(bytes).into_owned();

        commits.push(Commit {
            hash: text(hash),
            parents: text(parents).split_whitespace().map(str::to_string).collect(),
            subject: text(subject),
            refs: parse_refs(&text(decoration)),
            author: text(author),
            timestamp: text(timestamp).trim().parse().unwrap_or(0),
        });
    }

    commits
}

/// Reads up to `limit` commits of history from the repository at `root`.
///
/// Asks for branches, remotes and tags rather than `--all`. `--all` also
/// walks private ref namespaces that tools write into, such as the
/// checkpoint refs some editors keep, and those commits would appear in
/// the graph as unrelated roots.
///
/// Returns an empty list when there is no repository, no commits yet, or
/// no git on the machine.
pub fn load_history(root: Option<&Path>, limit: usize) -> Vec<Commit> {
    let format = format!(
        "--format=%H%x1f%P%x1f%s%x1f%D%x1f%an%x1f%at%x{:02x}",
        RECORD
    );

    let mut cmd = std::process::Command::new("git");
    cmd.arg("log")
        .arg("--branches")
        .arg("--remotes")
        .arg("--tags")
        .arg("--date-order")
        .arg("--decorate=full")
        .arg(format!("--max-count={limit}"))
        .arg(format);
    if let Some(dir) = root {
        cmd.current_dir(dir);
    }

    match cmd.output() {
        Ok(out) if out.status.success() => parse_log(&out.stdout),
        _ => Vec::new(),
    }
}

/// Runs [`load_history`] off the UI thread.
pub async fn load_history_async(root: Option<PathBuf>) -> Vec<Commit> {
    tokio::task::spawn_blocking(move || load_history(root.as_deref(), HISTORY_LIMIT))
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

    fn record(hash: &str, parents: &str, subject: &str, decoration: &str) -> Vec<u8> {
        format!("{hash}\x1f{parents}\x1f{subject}\x1f{decoration}\x1fAda\x1f1700000000\x1e\n")
            .into_bytes()
    }

    #[test]
    fn test_parse_log_keeps_all_six_fields() {
        let commits = parse_log(&record("abcdef1234", "p1", "do thing", ""));
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].hash, "abcdef1234");
        assert_eq!(commits[0].short_hash(), "abcdef1");
        assert_eq!(commits[0].parents, vec!["p1"]);
        assert_eq!(commits[0].subject, "do thing");
        assert_eq!(commits[0].author, "Ada");
        assert_eq!(commits[0].timestamp, 1_700_000_000);
    }

    #[test]
    fn test_parse_log_merge_keeps_parent_order() {
        let commits = parse_log(&record("m", "first second", "merge", ""));
        assert!(commits[0].is_merge());
        assert_eq!(commits[0].parents, vec!["first", "second"]);
    }

    #[test]
    fn test_parse_log_root_commit_has_no_parents() {
        let commits = parse_log(&record("r", "", "init", ""));
        assert!(commits[0].parents.is_empty());
        assert!(!commits[0].is_merge());
    }

    #[test]
    fn test_parse_log_newline_stays_out_of_next_hash() {
        let mut out = record("aaa", "", "one", "");
        out.extend(record("bbb", "aaa", "two", ""));
        let commits = parse_log(&out);
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[1].hash, "bbb");
    }

    #[test]
    fn test_parse_refs_kinds_are_told_apart() {
        let refs =
            parse_refs("HEAD -> refs/heads/main, tag: refs/tags/v1.0, refs/remotes/origin/main");
        assert_eq!(refs.len(), 3);
        assert_eq!(refs[0].kind, RefKind::LocalBranch);
        assert!(refs[0].is_head);
        assert_eq!(refs[0].name, "main");
        assert_eq!(refs[1].kind, RefKind::Tag);
        assert_eq!(refs[1].name, "v1.0");
        assert_eq!(refs[2].kind, RefKind::RemoteBranch);
        assert_eq!(refs[2].name, "origin/main");
    }

    #[test]
    fn test_parse_refs_slash_branch_is_not_remote() {
        let refs = parse_refs("refs/heads/feature/x, refs/remotes/origin/x");
        assert_eq!(refs[0].kind, RefKind::LocalBranch);
        assert_eq!(refs[0].name, "feature/x");
        assert_eq!(refs[1].kind, RefKind::RemoteBranch);
    }

    #[test]
    fn test_parse_refs_detached_head() {
        let refs = parse_refs("HEAD");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].kind, RefKind::Head);
        assert!(refs[0].is_head);
    }

    #[test]
    fn test_parse_log_subject_with_commas_and_arrows() {
        let commits = parse_log(&record("h", "", "fix a, b -> c", "refs/heads/main"));
        assert_eq!(commits[0].subject, "fix a, b -> c");
        assert_eq!(commits[0].refs.len(), 1);
    }

    #[test]
    fn test_parse_log_missing_fields_skipped() {
        assert!(parse_log(b"abc\x1fparent\x1e\n").is_empty());
    }

    #[test]
    fn test_parse_log_empty_output() {
        assert!(parse_log(b"").is_empty());
    }

    #[test]
    fn test_load_history_empty_outside_repository() -> Result<(), Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir().join(format!("pinel-nogit-{}", std::process::id()));
        std::fs::create_dir_all(&dir)?;
        let commits = load_history(Some(&dir), 10);
        std::fs::remove_dir_all(&dir)?;
        assert!(commits.is_empty());
        Ok(())
    }
}
