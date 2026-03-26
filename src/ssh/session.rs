use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[derive(Debug, Clone)]
pub struct RemoteEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
}

pub struct SshSession {
    target: String,
}

impl SshSession {
    pub fn connect(target: &str) -> Result<Self> {
        let session = Self {
            target: target.to_string(),
        };

        session.run_ssh("true")?;
        Ok(session)
    }

    pub fn canonicalize(&self, path: &str) -> Result<PathBuf> {
        let output = self.run_ssh_capture(&format!("cd {} && pwd -P", shell_quote(path)))?;
        let canonical = output.trim();

        if canonical.is_empty() {
            bail!("remote path resolved to an empty value");
        }

        Ok(PathBuf::from(canonical))
    }

    pub fn read_dir(&self, path: &Path) -> Result<Vec<RemoteEntry>> {
        let remote_path = path.to_string_lossy();
        let script = format!(
            "cd {} && \
             for item in .?* *; do \
                 [ \"$item\" = '.?*' ] && [ ! -e \"$item\" ] && continue; \
                 [ \"$item\" = '*' ] && [ ! -e \"$item\" ] && continue; \
                 [ \"$item\" = '.' ] && continue; \
                 [ \"$item\" = '..' ] && continue; \
                 if [ -d \"$item\" ]; then \
                     printf 'd\\0%s\\0' \"$item\"; \
                 else \
                     printf 'f\\0%s\\0' \"$item\"; \
                 fi; \
             done",
            shell_quote(&remote_path)
        );

        let output = self.run_ssh_bytes(&script)?;
        let mut fields = output.stdout.split(|byte| *byte == 0);
        let mut entries = Vec::new();

        while let Some(kind) = fields.next() {
            if kind.is_empty() {
                continue;
            }

            let Some(name) = fields.next() else {
                bail!("malformed remote directory listing");
            };

            let name = String::from_utf8(name.to_vec())
                .context("remote directory entry was not valid UTF-8")?;
            let is_dir = kind == b"d";

            entries.push(RemoteEntry {
                path: path.join(&name),
                name,
                is_dir,
            });
        }

        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        Ok(entries)
    }

    pub fn download_file(&self, remote_path: &Path, local_path: &Path) -> Result<()> {
        let remote_spec = format!(
            "{}:{}",
            self.target,
            shell_quote(&remote_path.to_string_lossy())
        );

        let output = Command::new("scp")
            .arg(remote_spec)
            .arg(local_path)
            .output()
            .context("failed to launch scp for remote download")?;

        ensure_success(output, "scp download failed")
    }

    pub fn upload_file(&self, local_path: &Path, remote_path: &Path) -> Result<()> {
        let remote_spec = format!(
            "{}:{}",
            self.target,
            shell_quote(&remote_path.to_string_lossy())
        );

        let output = Command::new("scp")
            .arg(local_path)
            .arg(remote_spec)
            .output()
            .context("failed to launch scp for remote upload")?;

        ensure_success(output, "scp upload failed")
    }

    fn run_ssh(&self, remote_command: &str) -> Result<()> {
        let output = self.run_ssh_bytes(remote_command)?;
        ensure_success(output, "ssh command failed")
    }

    fn run_ssh_capture(&self, remote_command: &str) -> Result<String> {
        let output = self.run_ssh_bytes(remote_command)?;
        ensure_success_with_stdout(output, "ssh command failed")
    }

    fn run_ssh_bytes(&self, remote_command: &str) -> Result<Output> {
        Command::new("ssh")
            .arg(&self.target)
            .arg(remote_command)
            .output()
            .with_context(|| format!("failed to launch ssh for {}", self.target))
    }
}

fn ensure_success(output: Output, context: &str) -> Result<()> {
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        bail!("{context}");
    }

    bail!("{context}: {stderr}");
}

fn ensure_success_with_stdout(output: Output, context: &str) -> Result<String> {
    if output.status.success() {
        return String::from_utf8(output.stdout).context("ssh output was not valid UTF-8");
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        bail!("{context}");
    }

    bail!("{context}: {stderr}");
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
