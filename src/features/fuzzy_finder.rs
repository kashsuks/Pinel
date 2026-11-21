use std::{
    fs,
    path::{Path, PathBuf},
};

/// A single file entry produced by scanning a directory.
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// Absolute path of the file.
    pub path: PathBuf,
    /// Path relative to the scanned root, shown in the finder.
    pub display_name: String,
}

pub struct FuzzyFinder {
    pub open: bool,
    pub input: String,
    pub current_folder: Option<PathBuf>,
    all_files: Vec<FileEntry>,
    pub filtered_files: Vec<FileEntry>,
    pub selected_index: usize,
    pub preview_cache: Option<(PathBuf, String)>, // Cached preview: (path that was loaded, file content string)
    pub input_id: iced::widget::Id,
}

impl Default for FuzzyFinder {
    fn default() -> Self {
        Self {
            open: false,
            input: String::new(),
            current_folder: None,
            all_files: Vec::new(),
            filtered_files: Vec::new(),
            selected_index: 0,
            preview_cache: None,
            input_id: iced::widget::Id::unique(),
        }
    }
}

const IGNORED_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    ".DS_Store",
    "__pycache__",
    ".claude",
    ".vscode",
    "dist",
    "build",
    ".next",
];

impl FuzzyFinder {
    pub fn toggle(&mut self) {
        self.open = !self.open;
        if self.open {
            self.input.clear();
            self.filtered_files = self.all_files.clone();
            self.selected_index = 0;
            self.preview_cache = None;
        }
    }

    pub fn close(&mut self) {
        self.open = false;
        self.input.clear();
        self.filtered_files = self.all_files.clone();
        self.selected_index = 0;
        self.preview_cache = None;
    }

    /// Point the finder at a workspace root using an already scanned file list.
    ///
    /// When the finder is open the current query is re-applied and the
    /// selection clamped to the new results; otherwise the results are reset
    /// to the full list.
    pub fn set_folder_files(&mut self, folder_path: PathBuf, files: Vec<FileEntry>) {
        self.current_folder = Some(folder_path);
        self.all_files = files;
        if self.open {
            let selected = self.selected_index;
            self.filter();
            self.selected_index = selected.min(self.filtered_files.len().saturating_sub(1));
        } else {
            self.filtered_files = self.all_files.clone();
            self.selected_index = 0;
        }
    }

    pub fn filter(&mut self) {
        if self.input.is_empty() {
            self.filtered_files = self.all_files.clone();
        } else {
            let input_lower = self.input.to_lowercase();

            let mut scored: Vec<(FileEntry, i32)> = self
                .all_files
                .iter()
                .filter_map(|file| {
                    let score = fuzzy_match(&file.display_name.to_lowercase(), &input_lower);
                    if score > 0 {
                        Some((file.clone(), score))
                    } else {
                        None
                    }
                })
                .collect();

            scored.sort_by(|(a, a_score), (b, b_score)| {
                b_score.cmp(a_score).then_with(|| a.display_name.cmp(&b.display_name))
            });

            self.filtered_files = scored.into_iter().map(|(file, _)| file).collect();
        }
        self.selected_index = 0;
        self.preview_cache = None;
    }

    pub fn navigate(&mut self, delta: i32) {
        let count = self.filtered_files.len();
        if count == 0 {
            return;
        }
        let current = self.selected_index as i32;
        self.selected_index = (current + delta).rem_euclid(count as i32) as usize;
        self.update_preview();
    }

    pub fn select(&mut self) -> Option<PathBuf> {
        let path = self.filtered_files.get(self.selected_index).map(|f| f.path.clone());
        self.close();
        path
    }

    pub fn update_preview(&mut self) {
        let Some(entry) = self.filtered_files.get(self.selected_index) else {
            self.preview_cache = None;
            return;
        };
        if let Some((cached_path, _)) = &self.preview_cache {
            if cached_path == &entry.path {
                return;
            }
        }
        // Read first ~200 lines for preview (no need to load huge files)
        let content = fs::read_to_string(&entry.path)
            .unwrap_or_else(|_| String::from("[binary or unreadable file]"));
        let truncated: String = content.lines().take(200).collect::<Vec<_>>().join("\n");
        self.preview_cache = Some((entry.path.clone(), truncated));
    }

    #[allow(dead_code)]
    pub fn selected_extension(&self) -> &str {
        self.filtered_files
            .get(self.selected_index)
            .and_then(|f| f.path.extension())
            .and_then(|e| e.to_str())
            .unwrap_or("")
    }
}

/// Recursively lists the files under `root`, skipping hidden entries and
/// ignored directories. Symlinked directories are never followed.
///
/// This walks the whole workspace, so call it off the UI thread.
pub fn scan_folder(root: &Path) -> Vec<FileEntry> {
    scan_directory(root, root)
}

fn scan_directory(dir: &Path, root: &Path) -> Vec<FileEntry> {
    let mut files = Vec::new();

    let Ok(entries) = fs::read_dir(dir) else {
        return files;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        let Some(name) = path.file_name() else {
            continue;
        };
        let name_str = name.to_string_lossy();

        // Skip hidden files/dirs and ignored directories
        if name_str.starts_with('.') {
            continue;
        }
        if IGNORED_DIRS.contains(&name_str.as_ref()) {
            continue;
        }

        // `file_type` does not follow symlinks, so a link back up the tree
        // cannot make this recurse forever.
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let is_file = file_type.is_file() || (file_type.is_symlink() && path.is_file());

        if is_file {
            let display_name =
                path.strip_prefix(root).unwrap_or(&path).to_string_lossy().to_string();
            files.push(FileEntry { path, display_name });
        } else if file_type.is_dir() {
            files.extend(scan_directory(&path, root));
        }
    }

    files.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    files
}

/// Fuzzy matching algorithm for the inbuilt fuzzy finder
///
/// # Arguments
///
/// - `text` (`&str`) - Input text to fuzzy find provided by user.
/// - `pattern` (`&str`) - String pattern that it follows.
///
/// # Returns
///
/// - `i32` - Final results for the fuzzy finder.
fn fuzzy_match(text: &str, pattern: &str) -> i32 {
    if pattern.is_empty() {
        return 1;
    }

    let mut score: i32 = 0;
    let mut pattern_idx = 0;
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();

    for (i, &ch) in text_chars.iter().enumerate() {
        if pattern_idx < pattern_chars.len() && ch == pattern_chars[pattern_idx] {
            score += 100;

            if pattern_idx > 0 && i > 0 && text_chars[i - 1] == pattern_chars[pattern_idx - 1] {
                score += 50;
            }

            if i == 0
                || text_chars[i - 1] == '/'
                || text_chars[i - 1] == '_'
                || text_chars[i - 1] == '.'
            {
                score += 30;
            }

            pattern_idx += 1;
        }
    }

    if pattern_idx == pattern_chars.len() {
        score
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, fs};

    use super::*;

    #[test]
    fn test_scan_folder_lists_nested_files() -> Result<(), Box<dyn Error>> {
        let dir = tempfile::tempdir()?;
        let root = dir.path();
        fs::create_dir_all(root.join("a").join("b"))?;
        fs::write(root.join("a").join("b").join("c.txt"), "c")?;

        let files = scan_folder(root);

        assert!(files.iter().any(|f| f.path == root.join("a").join("b").join("c.txt")));
        Ok(())
    }

    #[test]
    fn test_scan_folder_skips_hidden_and_ignored_dirs() -> Result<(), Box<dyn Error>> {
        let dir = tempfile::tempdir()?;
        let root = dir.path();
        fs::create_dir_all(root.join(".hidden"))?;
        fs::write(root.join(".hidden").join("x"), "x")?;
        fs::create_dir_all(root.join("node_modules"))?;
        fs::write(root.join("node_modules").join("y"), "y")?;
        fs::write(root.join("kept.txt"), "k")?;

        let files = scan_folder(root);

        assert!(files.iter().any(|f| f.path == root.join("kept.txt")));
        assert!(!files.iter().any(|f| f.path.starts_with(root.join(".hidden"))));
        assert!(!files.iter().any(|f| f.path.starts_with(root.join("node_modules"))));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_scan_folder_symlink_loop_terminates() -> Result<(), Box<dyn Error>> {
        let dir = tempfile::tempdir()?;
        let root = dir.path();
        fs::create_dir_all(root.join("dir"))?;
        fs::write(root.join("dir").join("real.txt"), "r")?;
        fs::write(root.join("top.txt"), "t")?;
        std::os::unix::fs::symlink(root, root.join("dir").join("loop"))?;

        let files = scan_folder(root);

        assert!(files.iter().any(|f| f.path == root.join("top.txt")));
        assert!(files.iter().any(|f| f.path == root.join("dir").join("real.txt")));
        assert!(!files.iter().any(|f| f.path.starts_with(root.join("dir").join("loop"))));
        assert_eq!(files.len(), 2);
        Ok(())
    }
}
