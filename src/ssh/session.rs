use anyhow::{Context, Result};
use ssh2::{FileStat, Session, Sftp};
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct RemoteEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
}

pub struct SshSession {
    _session: Session,
    sftp: Sftp,
}

impl SshSession {
    pub fn connect(target: &str) -> Result<Self> {
        let (user, host) = split_target(target);
        let tcp = TcpStream::connect(format!("{host}:22"))
            .with_context(|| format!("failed to connect to {host}:22"))?;

        let mut session = Session::new().context("failed to create ssh session")?;
        session.set_tcp_stream(tcp);
        session.handshake().context("ssh handshake failed")?;
        session
            .userauth_agent(&user)
            .with_context(|| format!("ssh agent authentication failed for {user}@{host}"))?;

        let sftp = session.sftp().context("failed to open sftp subsystem")?;
        Ok(Self {
            _session: session,
            sftp,
        })
    }

    pub fn canonicalize(&self, path: &str) -> Result<PathBuf> {
        self.sftp
            .realpath(Path::new(path))
            .with_context(|| format!("failed to resolve remote path {path}"))
    }

    pub fn read_dir(&self, path: &Path) -> Result<Vec<RemoteEntry>> {
        let mut entries = self
            .sftp
            .readdir(path)
            .with_context(|| format!("failed to read remote directory {}", path.display()))?
            .into_iter()
            .filter_map(|(path, stat)| {
                let name = path.file_name()?.to_string_lossy().to_string();

                if name == "." || name == ".." {
                    return None;
                }

                Some(RemoteEntry {
                    path,
                    name,
                    is_dir: is_dir(&stat),
                })
            })
            .collect::<Vec<_>>();

        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        Ok(entries)
    }

    pub fn download_file(&self, remote_path: &Path, local_path: &Path) -> Result<()> {
        let mut remote_file = self
            .sftp
            .open(remote_path)
            .with_context(|| format!("failed to open remote file {}", remote_path.display()))?;

        let mut contents = Vec::new();
        remote_file.read_to_end(&mut contents)?;

        let mut local_file = File::create(local_path)
            .with_context(|| format!("failed to create local file {}", local_path.display()))?;
        local_file.write_all(&contents)?;

        Ok(())
    }

    pub fn upload_file(&self, local_path: &Path, remote_path: &Path) -> Result<()> {
        let mut local_file = File::open(local_path)
            .with_context(|| format!("failed to open local file {}", local_path.display()))?;

        let mut contents = Vec::new();
        local_file.read_to_end(&mut contents)?;

        let mut remote_file = self
            .sftp
            .create(remote_path)
            .with_context(|| format!("failed to write remote file {}", remote_path.display()))?;
        remote_file.write_all(&contents)?;

        Ok(())
    }
}

fn split_target(target: &str) -> (String, String) {
    match target.split_once('@') {
        Some((user, host)) => (user.to_string(), host.to_string()),
        None => (whoami::username(), target.to_string()),
    }
}

fn is_dir(stat: &FileStat) -> bool {
    const S_IFMT: u32 = 0o170000;
    const S_IFDIR: u32 = 0o040000;

    stat.perm
        .map(|perm| (perm & S_IFMT) == S_IFDIR)
        .unwrap_or(false)
}
