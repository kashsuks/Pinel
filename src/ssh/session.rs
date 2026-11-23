use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use anyhow::{bail, Context, Result};

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
        validate_target(target)?;

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
        let script = format!("cat -- {}", shell_quote(&remote_path.to_string_lossy()));
        let output = self.run_ssh_bytes(&script)?;
        let contents = ensure_success_bytes(output, "ssh download failed")?;

        crate::fs_util::write_atomic(local_path, contents)
            .with_context(|| format!("could not write {}", local_path.display()))
    }

    pub fn upload_file(&self, local_path: &Path, remote_path: &Path) -> Result<()> {
        let contents = std::fs::read(local_path)
            .with_context(|| format!("could not read {}", local_path.display()))?;
        let script = format!("cat > {}", shell_quote(&remote_path.to_string_lossy()));
        let output = self.run_ssh_with_stdin(&script, contents)?;

        ensure_success(output, "ssh upload failed")
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
        ssh_command(&self.target, remote_command)
            .output()
            .with_context(|| format!("failed to launch ssh for {}", self.target))
    }

    /// Runs `remote_command` and feeds `input` to its standard input
    fn run_ssh_with_stdin(&self, remote_command: &str, input: Vec<u8>) -> Result<Output> {
        let mut child = ssh_command(&self.target, remote_command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("failed to launch ssh for {}", self.target))?;

        let mut stdin = child.stdin.take().context("ssh stdin was not available")?;
        let writer = std::thread::spawn(move || {
            let _ = stdin.write_all(&input);
        });

        let output = child.wait_with_output().context("failed while waiting for ssh")?;
        let _ = writer.join();
        Ok(output)
    }
}

fn ssh_command(target: &str, remote_command: &str) -> Command {
    let mut cmd = Command::new("ssh");
    cmd.arg("--").arg(target).arg(remote_command);
    cmd
}

fn validate_target(target: &str) -> Result<()> {
    if target.trim().is_empty() {
        bail!("ssh target is empty");
    }
    if target.starts_with('-') {
        bail!("ssh target {target:?} must not start with '-'");
    }
    if target.chars().any(|c| c.is_whitespace() || c.is_control()) {
        bail!("ssh target {target:?} must not contain whitespace or control characters");
    }
    Ok(())
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

fn ensure_success_bytes(output: Output, context: &str) -> Result<Vec<u8>> {
    if output.status.success() {
        return Ok(output.stdout);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(cmd: &Command) -> Vec<String> {
        cmd.get_args().map(|a| a.to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn ssh_command_puts_double_dash_before_the_target() {
        let cmd = ssh_command("user@host", "true");

        assert_eq!(cmd.get_program(), "ssh");
        assert_eq!(args(&cmd), ["--", "user@host", "true"]);
    }

    #[test]
    fn ssh_command_keeps_an_option_looking_target_as_a_plain_argument() {
        // validate_target rejects this first, but the command must be safe on its own too.
        let cmd = ssh_command("-oProxyCommand=touch /tmp/pwned", "true");

        let args = args(&cmd);
        assert_eq!(args[0], "--");
        assert_eq!(args[1], "-oProxyCommand=touch /tmp/pwned");
    }

    #[test]
    fn validate_target_accepts_normal_targets() {
        for target in [
            "host",
            "user@host",
            "user@192.168.1.5",
            "user@[::1]",
            "ssh://user@host:2222",
            "my-alias",
        ] {
            assert!(
                validate_target(target).is_ok(),
                "{target} should be accepted"
            );
        }
    }

    #[test]
    fn validate_target_rejects_empty_option_like_and_whitespace_targets() {
        for target in [
            "",
            "   ",
            "-oProxyCommand=id",
            "-v",
            "host name",
            "host\nname",
            "host\0",
        ] {
            assert!(
                validate_target(target).is_err(),
                "{target:?} should be rejected"
            );
        }
    }

    #[test]
    fn connect_rejects_a_bad_target_before_running_ssh() {
        let error = SshSession::connect("-oProxyCommand=id").err().expect("must fail");

        assert!(error.to_string().contains("must not start with '-'"));
    }

    #[test]
    fn shell_quote_wraps_and_escapes_single_quotes() {
        assert_eq!(shell_quote("plain"), "'plain'");
        assert_eq!(shell_quote("a b"), "'a b'");
        assert_eq!(shell_quote("it's"), r#"'it'\''s'"#);
        assert_eq!(shell_quote("$(id) `id` ; &"), "'$(id) `id` ; &'");
    }

    // The tests below talk to a real SSH server, so they only run on request:
    //
    //   PINEL_TEST_SSH_TARGET=user@host cargo test --bin pinel ssh:: -- --ignored
    //
    // The target must accept non-interactive (key based) logins. They only
    // create and delete files inside a fresh temporary directory on the host.

    fn live_session() -> Option<SshSession> {
        let target = std::env::var("PINEL_TEST_SSH_TARGET").ok()?;
        Some(SshSession::connect(&target).expect("connect to PINEL_TEST_SSH_TARGET"))
    }

    fn remote_tempdir(session: &SshSession) -> PathBuf {
        let out = session.run_ssh_capture("mktemp -d").expect("mktemp -d on remote");
        PathBuf::from(out.trim())
    }

    fn sample_bytes() -> Vec<u8> {
        // Every byte value, repeated, to prove the transfer is binary safe.
        (0..=255u8).cycle().take(256 * 40).collect()
    }

    #[test]
    #[ignore = "needs PINEL_TEST_SSH_TARGET"]
    fn live_upload_then_download_roundtrips_tricky_file_names() {
        let Some(session) = live_session() else {
            panic!("set PINEL_TEST_SSH_TARGET")
        };
        let remote_dir = remote_tempdir(&session);
        let local = tempfile::tempdir().unwrap();
        let data = sample_bytes();

        for name in [
            "plain.bin",
            "with space.bin",
            "it's.bin",
            "dollar$(id).bin",
            "semi;colon&amp.bin",
            "uni\u{e9}.bin",
        ] {
            let source = local.path().join("source");
            std::fs::write(&source, &data).unwrap();
            let remote = remote_dir.join(name);

            session
                .upload_file(&source, &remote)
                .unwrap_or_else(|e| panic!("upload {name}: {e:#}"));

            let back = local.path().join("back");
            session
                .download_file(&remote, &back)
                .unwrap_or_else(|e| panic!("download {name}: {e:#}"));
            assert_eq!(
                std::fs::read(&back).unwrap(),
                data,
                "{name} changed in transit"
            );
        }

        let listing = session.read_dir(&remote_dir).unwrap();
        assert_eq!(listing.len(), 6);
        session
            .run_ssh(&format!(
                "rm -rf {}",
                shell_quote(&remote_dir.to_string_lossy())
            ))
            .unwrap();
    }

    #[test]
    #[ignore = "needs PINEL_TEST_SSH_TARGET"]
    fn live_upload_replaces_an_existing_longer_file() {
        let Some(session) = live_session() else {
            panic!("set PINEL_TEST_SSH_TARGET")
        };
        let remote_dir = remote_tempdir(&session);
        let local = tempfile::tempdir().unwrap();
        let remote = remote_dir.join("file.txt");

        let long = local.path().join("long");
        std::fs::write(&long, "a much longer original body").unwrap();
        session.upload_file(&long, &remote).unwrap();
        let short = local.path().join("short");
        std::fs::write(&short, "short").unwrap();
        session.upload_file(&short, &remote).unwrap();

        let back = local.path().join("back");
        session.download_file(&remote, &back).unwrap();
        assert_eq!(std::fs::read_to_string(&back).unwrap(), "short");
        session
            .run_ssh(&format!(
                "rm -rf {}",
                shell_quote(&remote_dir.to_string_lossy())
            ))
            .unwrap();
    }

    #[test]
    #[ignore = "needs PINEL_TEST_SSH_TARGET"]
    fn live_failed_download_reports_the_error_and_leaves_no_local_file() {
        let Some(session) = live_session() else {
            panic!("set PINEL_TEST_SSH_TARGET")
        };
        let remote_dir = remote_tempdir(&session);
        let local = tempfile::tempdir().unwrap();
        let target = local.path().join("never-created");

        let error = session
            .download_file(&remote_dir.join("missing.txt"), &target)
            .expect_err("missing file must fail");

        assert!(
            error.to_string().contains("ssh download failed"),
            "{error:#}"
        );
        assert!(!target.exists());
        session
            .run_ssh(&format!(
                "rm -rf {}",
                shell_quote(&remote_dir.to_string_lossy())
            ))
            .unwrap();
    }

    #[test]
    #[ignore = "needs PINEL_TEST_SSH_TARGET"]
    fn live_failed_upload_reports_the_error() {
        let Some(session) = live_session() else {
            panic!("set PINEL_TEST_SSH_TARGET")
        };
        let remote_dir = remote_tempdir(&session);
        let local = tempfile::tempdir().unwrap();
        let source = local.path().join("source");
        std::fs::write(&source, vec![7u8; 2_000_000]).unwrap();

        let error = session
            .upload_file(&source, &remote_dir.join("no-such-dir").join("file.txt"))
            .expect_err("missing directory must fail");

        assert!(error.to_string().contains("ssh upload failed"), "{error:#}");
        session
            .run_ssh(&format!(
                "rm -rf {}",
                shell_quote(&remote_dir.to_string_lossy())
            ))
            .unwrap();
    }

    #[test]
    #[ignore = "needs PINEL_TEST_SSH_TARGET"]
    fn live_large_file_roundtrips() {
        let Some(session) = live_session() else {
            panic!("set PINEL_TEST_SSH_TARGET")
        };
        let remote_dir = remote_tempdir(&session);
        let local = tempfile::tempdir().unwrap();
        let data: Vec<u8> = (0..20_000_000u32).map(|i| (i % 251) as u8).collect();
        let source = local.path().join("source");
        std::fs::write(&source, &data).unwrap();
        let remote = remote_dir.join("big.bin");

        session.upload_file(&source, &remote).unwrap();
        let back = local.path().join("back");
        session.download_file(&remote, &back).unwrap();

        assert!(
            std::fs::read(&back).unwrap() == data,
            "20 MB file changed in transit"
        );
        session
            .run_ssh(&format!(
                "rm -rf {}",
                shell_quote(&remote_dir.to_string_lossy())
            ))
            .unwrap();
    }
}
