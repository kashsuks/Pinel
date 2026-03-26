/// This file provides code related to the File Tree and
/// sidebar. Toggling a folder, selecting files/folder, and others
/// are part of this file
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// This enum provides the values needed for a file to be registered and accessed
///
/// # Variants
///
/// - `File { path, name }` - The path and name of the file being accessed.
/// - `Directory { path, name, children }` - Information related to a directory such as children.
///
/// # Examples
///
/// ```
/// use crate::...;
///
/// let fileentry = FileEntry::File;
/// match fileentry {
///     FileEntry::File { path, name } => handle_fields,
///     FileEntry::Directory { path, name, children } => handle_fields,
/// }
/// ```
#[derive(Debug, Clone)]
pub enum FileEntry {
    File {
        path: PathBuf,
        name: String,
    },
    Directory {
        path: PathBuf,
        name: String,
        children: Vec<FileEntry>, // A vector containing elements of type FileEntry, meaning either files or other directories, like I said above
    },
}

#[derive(Debug, Clone)]
pub struct FileTree {
    pub root: PathBuf,
    pub entries: Vec<FileEntry>,
    pub expanded: HashSet<PathBuf>,
    // Storing only expanded ones, not collapsed ones to save memory
    // Collapsed ones are simply all of those that are not expanded
    pub selected: Option<PathBuf>,
}

impl FileTree {
    pub fn new(root: PathBuf) -> Self {
        let entries = scan_directory(&root);
        Self {
            root,
            entries,
            expanded: HashSet::new(),
            selected: None,
        }
    }

    pub fn toggle_folder(&mut self, path: &Path) {
        if self.expanded.contains(path) {
            self.expanded.remove(path);
        } else {
            self.expanded.insert(path.to_path_buf());
            populate_children(&mut self.entries, path); // Lazily load this folder's contents
        }
    }

    pub fn is_expanded(&self, path: &Path) -> bool {
        self.expanded.contains(path)
    }

    pub fn select(&mut self, path: PathBuf) {
        self.selected = Some(path);
    }

    pub fn refresh(&mut self) {
        self.entries = scan_directory(&self.root);
        let mut expanded: Vec<PathBuf> = self.expanded.iter().cloned().collect();
        expanded.sort_by_key(|p| p.components().count());
        for path in expanded {
            populate_children(&mut self.entries, &path);
        }
    }
}

const IGNORED_DIRS: &[&str] = &[".git", "node_modules", "target", ".DS_Store", "__pycache__"];

fn scan_directory(path: &Path) -> Vec<FileEntry> {
    let mut entries = Vec::new(); // An empty vector of entires

    let Ok(read_dir) = fs::read_dir(path) else {
        return entries; // If the result is an error, return the empty vector
    };

    for entry in read_dir.flatten() {
        let entry_path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if IGNORED_DIRS.contains(&name.as_str()) {
            continue;
        }

        if entry_path.is_dir() {
            entries.push(FileEntry::Directory {
                path: entry_path,
                name,
                children: Vec::new(),
            });
        } else {
            entries.push(FileEntry::File {
                path: entry_path,
                name,
            });
        }
    }

    entries.sort_by(|a, b| match (a, b) {
        (FileEntry::Directory { name: name_a, .. }, FileEntry::Directory { name: name_b, .. }) => {
            name_a.to_lowercase().cmp(&name_b.to_lowercase())
        }
        (FileEntry::File { name: name_a, .. }, FileEntry::File { name: name_b, .. }) => {
            name_a.to_lowercase().cmp(&name_b.to_lowercase())
        }
        (FileEntry::Directory { .. }, FileEntry::File { .. }) => std::cmp::Ordering::Less,
        (FileEntry::File { .. }, FileEntry::Directory { .. }) => std::cmp::Ordering::Greater,
    });

    return entries;
}

/// Lazily load the children of a folder by only storing parent dirs
/// until they are being accessed
///
/// # Arguments
///
/// - `entries` (`&mut Vec<FileEntry>`) - Related parent directory entries.
/// - `target` (`&Path`) - Final target child file.
fn populate_children(entries: &mut Vec<FileEntry>, target: &Path) {
    for entry in entries.iter_mut() {
        if let FileEntry::Directory { path, children, .. } = entry {
            if path == target {
                if children.is_empty() {
                    *children = scan_directory(path);
                }
                return;
            }
            populate_children(children, target);
        }
    }
}
