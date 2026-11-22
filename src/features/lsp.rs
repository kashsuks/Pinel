//! LSP support using iced-code-editor's lsp-process feature.
//!
//! Provides hover documentation, auto-completion, and go-to-definition
//! for supported language servers.

use std::{
    path::{Path, PathBuf},
    sync::mpsc,
};

use iced_code_editor::{LspClient, LspEvent, LspProcessClient};

/// Files and folders whose presence marks a directory as a project root.
const ROOT_MARKERS: &[&str] = &[
    ".git",
    "Cargo.toml",
    "go.mod",
    "pyproject.toml",
    "setup.py",
    "pyrightconfig.json",
    "package.json",
    "tsconfig.json",
    ".luarc.json",
];

pub struct LspManager {
    sender: mpsc::Sender<LspEvent>,
    receiver: Option<mpsc::Receiver<LspEvent>>,
    workspace_root: Option<PathBuf>,
}

impl LspManager {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver: Some(receiver),
            workspace_root: None,
        }
    }

    pub fn set_workspace_root(&mut self, root: PathBuf) {
        self.workspace_root = Some(root);
    }

    /// Creates a new LSP process client for the given server key.
    /// The client should be attached to a CodeEditor via `editor.attach_lsp()`.
    pub fn create_client(
        &self,
        server_key: &str,
        root_hint: Option<&Path>,
    ) -> Result<Box<dyn LspClient>, String> {
        let root_uri = self.root_uri(root_hint);
        let client = LspProcessClient::new_with_server(&root_uri, self.sender.clone(), server_key)
            .map_err(|e| format!("Failed to start LSP server '{}': {}", server_key, e))?;
        Ok(Box::new(client))
    }

    pub fn drain_events(&mut self) -> Vec<LspEvent> {
        let mut events = Vec::new();
        if let Some(ref receiver) = self.receiver {
            loop {
                match receiver.try_recv() {
                    Ok(event) => events.push(event),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        self.receiver = None;
                        break;
                    },
                }
            }
        }
        events
    }

    fn root_uri(&self, root_hint: Option<&Path>) -> String {
        let hint_dir = root_hint.and_then(|path| {
            if path.is_dir() {
                Some(path)
            } else {
                path.parent()
            }
        });
        let root_dir = match hint_dir {
            Some(dir) => Some(project_root(
                dir,
                self.workspace_root.as_deref(),
                dirs::home_dir().as_deref(),
                &has_root_marker,
            )),
            None => self.workspace_root.clone(),
        };

        match root_dir {
            Some(dir) => document_uri(&dir),
            None => "file:///".to_string(),
        }
    }
}

impl Default for LspManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Whether `dir` holds one of the [`ROOT_MARKERS`]. Uses `exists()` rather
/// than `is_file()` because `.git` is a file in worktrees and submodules.
fn has_root_marker(dir: &Path) -> bool {
    ROOT_MARKERS.iter().any(|marker| dir.join(marker).exists())
}

/// Whether `dir` is too broad to be a project root: the filesystem root (a
/// path with no parent), `home`, or any ancestor of `home`.
fn is_too_broad(dir: &Path, home: Option<&Path>) -> bool {
    dir.parent().is_none() || home.is_some_and(|home| home.starts_with(dir))
}

/// Picks the directory a language server should treat as the project root
/// for a file in `file_dir`. The process working directory is never used.
///
/// - If `file_dir` is inside `workspace_root` (the opened folder), the
///   directories from the opened folder down to `file_dir` are checked top
///   first, and the first one with a marker wins. That is the shallowest
///   marker, so an opened project stays the root even for files under its
///   `node_modules/` or `vendor/`, while a folder of projects gives the
///   project the file belongs to. With no marker the root is the opened
///   folder, or `file_dir` if the opened folder is too broad.
/// - Otherwise (no folder open, or the file is outside it), the walk goes up
///   from `file_dir`, nearest first, and returns the first directory with a
///   marker, else `file_dir`.
///
/// Too broad directories (the filesystem root, `home` and its ancestors) are
/// never picked for their markers, so a stray `~/.git` can't make the whole
/// home folder the root. A file directly in `home` with no folder open still
/// gets `home`, because `file_dir` is the fallback. The check is lexical, so
/// a `home` reached through a symlink isn't recognized.
///
/// `has_marker` reports whether a directory holds a root marker. It is
/// injected so the walk can be tested without touching the filesystem.
fn project_root(
    file_dir: &Path,
    workspace_root: Option<&Path>,
    home: Option<&Path>,
    has_marker: &dyn Fn(&Path) -> bool,
) -> PathBuf {
    if let Some(workspace) = workspace_root.filter(|ws| file_dir.starts_with(ws)) {
        let mut inside: Vec<&Path> =
            file_dir.ancestors().take_while(|dir| dir.starts_with(workspace)).collect();
        inside.reverse();
        if let Some(dir) =
            inside.into_iter().find(|dir| !is_too_broad(dir, home) && has_marker(dir))
        {
            return dir.to_path_buf();
        }
        if is_too_broad(workspace, home) {
            return file_dir.to_path_buf();
        }
        return workspace.to_path_buf();
    }

    file_dir
        .ancestors()
        .take_while(|dir| !is_too_broad(dir, home))
        .find(|dir| has_marker(dir))
        .unwrap_or(file_dir)
        .to_path_buf()
}

/// Returns the `file://` URI the editor uses for `path` when talking to a
/// language server. Characters such as spaces and `#` are percent-encoded.
///
/// Must match the URI given to `LspDocument::new` when a client is attached,
/// since responses such as document symbols are looked up by it.
///
/// `path` should be absolute. A relative path can't be turned into a proper
/// `file://` URI, so it falls back to `file://` followed by the raw,
/// unencoded path, with `\` turned into `/` on Windows.
pub fn document_uri(path: &Path) -> String {
    match url::Url::from_file_path(path) {
        Ok(url) => url.into(),
        Err(()) => {
            let raw = path.to_string_lossy();
            if cfg!(windows) {
                format!("file://{}", raw.replace('\\', "/"))
            } else {
                format!("file://{raw}")
            }
        },
    }
}

/// Provides structure for Inline diagnostic
///
/// # Fields
///
/// - `line` (`usize`) - The line that the user requires a diagnostic for.
/// - `severity` (`lsp_types`) - What severity is the error/warning in the users code.
/// - `message` (`String`) - The message that the LSP provides for inline diagnostic visuals.
///
/// # Examples
///
/// ```
/// use crate::lsp;
///
/// let s = InlineDiagnostic {
///     line: value,
///     severity: value,
///     message: value,
/// };
/// ```
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct InlineDiagnostic {
    pub line: usize,
    pub severity: lsp_types::DiagnosticSeverity,
    pub message: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DiagnosticUpdate {
    pub path: PathBuf,
    pub diagnostics: Vec<InlineDiagnostic>,
}

/// How many of `diagnostics` are errors and how many are warnings.
pub(crate) fn diagnostic_counts(
    diagnostics: &[crate::features::lsp::InlineDiagnostic],
) -> (usize, usize) {
    diagnostics.iter().fold((0, 0), |(errors, warnings), diagnostic| {
        match diagnostic.severity {
            lsp_types::DiagnosticSeverity::ERROR => (errors + 1, warnings),
            lsp_types::DiagnosticSeverity::WARNING => (errors, warnings + 1),
            _ => (errors, warnings),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    /// A marker check that reports a marker in exactly the directories listed.
    fn markers_at<'a>(dirs: &'a [&'a str]) -> impl Fn(&Path) -> bool + 'a {
        move |dir| dirs.iter().any(|marked| Path::new(marked) == dir)
    }

    #[test]
    fn test_project_root_workspace_with_marker_is_workspace() -> Result<(), Box<dyn Error>> {
        let has_marker = markers_at(&["/ws", "/ws/node_modules/react"]);

        let root = project_root(
            Path::new("/ws/node_modules/react"),
            Some(Path::new("/ws")),
            Some(Path::new("/home/me")),
            &has_marker,
        );

        assert_eq!(root, PathBuf::from("/ws"));
        Ok(())
    }

    #[test]
    fn test_project_root_workspace_without_marker_uses_shallowest_marker(
    ) -> Result<(), Box<dyn Error>> {
        let has_marker = markers_at(&["/code/rust/app"]);

        let root = project_root(
            Path::new("/code/rust/app/src"),
            Some(Path::new("/code")),
            Some(Path::new("/home/me")),
            &has_marker,
        );

        assert_eq!(root, PathBuf::from("/code/rust/app"));
        Ok(())
    }

    #[test]
    fn test_project_root_workspace_without_any_marker_is_workspace() -> Result<(), Box<dyn Error>> {
        let has_marker = markers_at(&[]);

        let root = project_root(
            Path::new("/ws/src/nested"),
            Some(Path::new("/ws")),
            Some(Path::new("/home/me")),
            &has_marker,
        );

        assert_eq!(root, PathBuf::from("/ws"));
        Ok(())
    }

    #[test]
    fn test_project_root_home_workspace_uses_markers() -> Result<(), Box<dyn Error>> {
        let has_marker = markers_at(&["/home/me", "/home/me/My Projects/app"]);

        let root = project_root(
            Path::new("/home/me/My Projects/app/src"),
            Some(Path::new("/home/me")),
            Some(Path::new("/home/me")),
            &has_marker,
        );

        assert_eq!(root, PathBuf::from("/home/me/My Projects/app"));
        Ok(())
    }

    #[test]
    fn test_project_root_home_workspace_without_marker_is_file_dir() -> Result<(), Box<dyn Error>> {
        // A stray marker in the home folder itself must not count.
        let has_marker = markers_at(&["/home/me"]);

        let root = project_root(
            Path::new("/home/me/My Projects/app/src"),
            Some(Path::new("/home/me")),
            Some(Path::new("/home/me")),
            &has_marker,
        );

        assert_eq!(root, PathBuf::from("/home/me/My Projects/app/src"));
        Ok(())
    }

    #[test]
    fn test_project_root_sibling_prefix_is_not_inside() -> Result<(), Box<dyn Error>> {
        let has_marker = markers_at(&["/ws", "/ws2"]);

        let root = project_root(
            Path::new("/ws2/x"),
            Some(Path::new("/ws")),
            Some(Path::new("/home/me")),
            &has_marker,
        );

        assert_eq!(root, PathBuf::from("/ws2"));
        Ok(())
    }

    #[test]
    fn test_project_root_outside_workspace_finds_nearest_marker() -> Result<(), Box<dyn Error>> {
        let has_marker = markers_at(&["/other", "/other/proj"]);

        let root = project_root(
            Path::new("/other/proj/src"),
            Some(Path::new("/ws")),
            Some(Path::new("/home/me")),
            &has_marker,
        );

        assert_eq!(root, PathBuf::from("/other/proj"));
        Ok(())
    }

    #[test]
    fn test_project_root_no_marker_is_file_dir() -> Result<(), Box<dyn Error>> {
        let has_marker = markers_at(&[]);

        let root = project_root(
            Path::new("/other/proj/src"),
            None,
            Some(Path::new("/home/me")),
            &has_marker,
        );

        assert_eq!(root, PathBuf::from("/other/proj/src"));
        Ok(())
    }

    #[test]
    fn test_project_root_stops_below_home() -> Result<(), Box<dyn Error>> {
        let has_marker = markers_at(&["/home/me"]);

        let root = project_root(
            Path::new("/home/me/scripts"),
            None,
            Some(Path::new("/home/me")),
            &has_marker,
        );

        assert_eq!(root, PathBuf::from("/home/me/scripts"));
        Ok(())
    }

    #[test]
    fn test_project_root_never_returns_filesystem_root() -> Result<(), Box<dyn Error>> {
        let has_marker = markers_at(&["/"]);

        let root = project_root(Path::new("/srv/files"), None, None, &has_marker);

        assert_eq!(root, PathBuf::from("/srv/files"));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_document_uri_percent_encodes_spaces() -> Result<(), Box<dyn Error>> {
        let uri = document_uri(Path::new("/home/me/My Projects/main.rs"));

        assert_eq!(uri, "file:///home/me/My%20Projects/main.rs");
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_document_uri_round_trips_through_file_uri_to_path() -> Result<(), Box<dyn Error>> {
        let path = Path::new("/home/me/My Projects/C# notes/main.rs");

        let uri = document_uri(path);
        let back = crate::features::definition::file_uri_to_path(&uri).ok_or("expected a path")?;

        assert!(
            !uri.contains('#'),
            "a raw '#' would start a fragment: {uri}"
        );
        assert_eq!(back, path);
        Ok(())
    }
}
