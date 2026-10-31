use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

/// Separates the fields within the record `git show` prints.
const FIELD: char = '\u{1f}';
/// Ends the record, so the stat summary after it can be told apart.
const RECORD: char = '\u{1e}';

/// Everything the hover card and the commit tab show about one commit.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommitDetail {
    pub hash: String,
    pub author: String,
    pub email: String,
    /// Author date, as seconds since the Unix epoch
    pub timestamp: i64,
    /// The author date in the local timezone, such as `October 29, 2026 at 10:58 PM`
    pub date: String,
    pub subject: String,
    pub body: String,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    /// `https://github.com/owner/repo/commit/<hash>`, when `origin` is on GitHub
    pub github_url: Option<String>,
}

/// The current time as seconds since the Unix epoch.
pub fn unix_now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64)
}

/// Words for how long ago `timestamp` was, such as `23 minutes ago`.
///
/// Anything under a minute, or in the future, reads as `just now`.
pub fn relative_time(now: i64, timestamp: i64) -> String {
    let secs = now - timestamp;
    if secs < 60 {
        return "just now".to_string();
    }
    let (count, unit) = match secs {
        s if s < 3_600 => (s / 60, "minute"),
        s if s < 86_400 => (s / 3_600, "hour"),
        s if s < 2_592_000 => (s / 86_400, "day"),
        s if s < 31_536_000 => (s / 2_592_000, "month"),
        s => (s / 31_536_000, "year"),
    };
    format!(
        "{count} {unit}{} ago",
        if count == 1 {
            ""
        } else {
            "s"
        }
    )
}

/// Turns a git remote URL into the GitHub page for `hash`
///
/// Understands the `git@github.com:o/r.git`, `ssh://git@github.com/o/r` and
/// `https://github.com/o/r` forms. Returns `None` for any other host
pub fn github_commit_url(remote: &str, hash: &str) -> Option<String> {
    let remote = remote.trim();
    let path = remote
        .strip_prefix("git@github.com:")
        .or_else(|| remote.strip_prefix("ssh://git@github.com/"))
        .or_else(|| remote.strip_prefix("https://github.com/"))
        .or_else(|| remote.strip_prefix("http://github.com/"))?;
    let path = path.trim_end_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    let (owner, repo) = path.split_once('/')?;
    if owner.is_empty() || repo.is_empty() || repo.contains('/') {
        return None;
    }
    Some(format!("https://github.com/{owner}/{repo}/commit/{hash}"))
}

/// Reads `(files changed, insertions, deletions)` from a `--shortstat` line.
///
/// A part git leaves out, like a commit with no deletions, counts as zero.
pub fn parse_shortstat(text: &str) -> (usize, usize, usize) {
    let mut counts = (0, 0, 0);
    for part in text.split(',') {
        let mut words = part.split_whitespace();
        let Some(count) = words.next().and_then(|n| n.parse::<usize>().ok()) else {
            continue;
        };
        match words.next() {
            Some(word) if word.starts_with("file") => counts.0 = count,
            Some(word) if word.starts_with("insertion") => counts.1 = count,
            Some(word) if word.starts_with("deletion") => counts.2 = count,
            _ => {},
        }
    }
    counts
}

/// Parses the output of the `git show` run by [`load_commit_detail`]
///
/// Returns `None` when the record is missing fields. The GitHub link is
/// left empty
pub fn parse_commit_detail(output: &[u8]) -> Option<CommitDetail> {
    let text = String::from_utf8_lossy(output);
    let (record, stat) = text.split_once(RECORD)?;

    let mut fields = record.trim_start().splitn(7, FIELD);
    let (hash, author, email, timestamp, date, subject, body) = (
        fields.next()?,
        fields.next()?,
        fields.next()?,
        fields.next()?,
        fields.next()?,
        fields.next()?,
        fields.next()?,
    );
    let (files_changed, insertions, deletions) = parse_shortstat(stat);

    Some(CommitDetail {
        hash: hash.to_string(),
        author: author.to_string(),
        email: email.to_string(),
        timestamp: timestamp.trim().parse().unwrap_or(0),
        // `%e` and `%l` pad with spaces, so squeeze them out
        date: date.split_whitespace().collect::<Vec<_>>().join(" "),
        subject: subject.to_string(),
        body: body.trim().to_string(),
        files_changed,
        insertions,
        deletions,
        github_url: None,
    })
}

fn git_command(root: Option<&Path>) -> std::process::Command {
    let mut cmd = std::process::Command::new("git");
    if let Some(dir) = root {
        cmd.current_dir(dir);
    }
    cmd
}

/// The URL of the `origin` remote, if there is one.
fn load_remote_url(root: Option<&Path>) -> Option<String> {
    let out = git_command(root).args(["remote", "get-url", "origin"]).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Reads the details of the commit `hash` from the repository at `root`.
///
/// Merges are measured against their first parent. Returns `None` when
/// there is no repository or git does not know the commit.
pub fn load_commit_detail(root: Option<&Path>, hash: &str) -> Option<CommitDetail> {
    let out = git_command(root)
        .args(["show", "-m", "--first-parent", "--shortstat"])
        .arg("--date=format-local:%B %e, %Y at %l:%M %p")
        .arg("--format=%H%x1f%an%x1f%ae%x1f%at%x1f%ad%x1f%s%x1f%b%x1e")
        .arg(hash)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }

    let mut detail = parse_commit_detail(&out.stdout)?;
    detail.github_url =
        load_remote_url(root).and_then(|remote| github_commit_url(&remote, &detail.hash));
    Some(detail)
}

/// runs [`load_commit_detail`] off the UI thread
pub async fn load_commit_detail_async(root: Option<PathBuf>, hash: String) -> Option<CommitDetail> {
    tokio::task::spawn_blocking(move || load_commit_detail(root.as_deref(), &hash))
        .await
        .ok()
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    const SAMPLE: &str = "abc123\x1fAda\x1fada@example.com\x1f1700000000\x1fOctober  5, 2026 at  9:03 PM\x1ffeat: add card\x1fFirst line.\n\nSecond line.\n\x1e\n\n 2 files changed, 9 insertions(+), 3 deletions(-)\n";

    #[test]
    fn test_parse_shortstat_all_parts() {
        assert_eq!(
            parse_shortstat(" 2 files changed, 9 insertions(+), 3 deletions(-)"),
            (2, 9, 3)
        );
    }

    #[test]
    fn test_parse_shortstat_missing_parts_count_as_zero() {
        assert_eq!(
            parse_shortstat(" 1 file changed, 4 insertions(+)"),
            (1, 4, 0)
        );
        assert_eq!(
            parse_shortstat(" 1 file changed, 2 deletions(-)"),
            (1, 0, 2)
        );
        assert_eq!(parse_shortstat(""), (0, 0, 0));
    }

    #[test]
    fn test_parse_commit_detail_reads_every_field() -> TestResult {
        let detail = parse_commit_detail(SAMPLE.as_bytes()).ok_or("no detail")?;
        assert_eq!(detail.hash, "abc123");
        assert_eq!(detail.author, "Ada");
        assert_eq!(detail.email, "ada@example.com");
        assert_eq!(detail.timestamp, 1_700_000_000);
        assert_eq!(detail.date, "October 5, 2026 at 9:03 PM");
        assert_eq!(detail.subject, "feat: add card");
        assert_eq!(detail.body, "First line.\n\nSecond line.");
        assert_eq!(
            (detail.files_changed, detail.insertions, detail.deletions),
            (2, 9, 3)
        );
        Ok(())
    }

    #[test]
    fn test_parse_commit_detail_empty_body_and_no_stat() -> TestResult {
        let text = "h\x1fA\x1fa@b.c\x1f5\x1fd\x1fsubject\x1f\x1e\n";
        let detail = parse_commit_detail(text.as_bytes()).ok_or("no detail")?;
        assert!(detail.body.is_empty());
        assert_eq!(detail.files_changed, 0);
        Ok(())
    }

    #[test]
    fn test_parse_commit_detail_truncated_record_is_none() {
        assert!(parse_commit_detail(b"abc\x1fAda\x1e").is_none());
        assert!(parse_commit_detail(b"").is_none());
    }

    #[test]
    fn test_github_commit_url_accepts_every_github_form() {
        let want = Some("https://github.com/o/r/commit/abc".to_string());
        for remote in [
            "git@github.com:o/r.git",
            "git@github.com:o/r",
            "ssh://git@github.com/o/r.git",
            "https://github.com/o/r.git",
            "https://github.com/o/r/",
            "https://github.com/o/r\n",
        ] {
            assert_eq!(github_commit_url(remote, "abc"), want, "{remote}");
        }
    }

    #[test]
    fn test_github_commit_url_rejects_other_hosts_and_junk() {
        for remote in [
            "git@gitlab.com:o/r.git",
            "https://example.com/o/r",
            "https://github.com/onlyowner",
            "https://github.com/o/r/extra",
            "",
        ] {
            assert_eq!(github_commit_url(remote, "abc"), None, "{remote}");
        }
    }

    #[test]
    fn test_relative_time_boundaries() {
        assert_eq!(relative_time(1000, 1000), "just now");
        assert_eq!(relative_time(1000, 2000), "just now");
        assert_eq!(relative_time(1060, 1000), "1 minute ago");
        assert_eq!(relative_time(1000 + 23 * 60, 1000), "23 minutes ago");
        assert_eq!(relative_time(1000 + 3_600, 1000), "1 hour ago");
        assert_eq!(relative_time(1000 + 2 * 86_400, 1000), "2 days ago");
        assert_eq!(relative_time(1000 + 40 * 86_400, 1000), "1 month ago");
        assert_eq!(relative_time(1000 + 800 * 86_400, 1000), "2 years ago");
    }
}
