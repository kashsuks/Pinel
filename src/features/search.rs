use std::{cmp::Reverse, path::PathBuf};

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub line_number: usize,
    pub line_content: String,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: PathBuf,
    pub file_name: String,
    pub matches: Vec<SearchMatch>,
}

pub fn search_workspace(root: &PathBuf, query: &str) -> Vec<SearchResult> {
    use std::fs;

    use ignore::WalkBuilder;

    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    let walker = WalkBuilder::new(root).hidden(true).git_ignore(true).git_global(true).build();

    for entry in walker.flatten() {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };

        let mut matches = Vec::new();
        for (line_idx, line) in content.lines().enumerate() {
            if line.to_lowercase().contains(&query_lower) {
                matches.push(SearchMatch {
                    line_number: line_idx + 1,
                    line_content: line.to_string(),
                });
            }
        }

        if !matches.is_empty() {
            results.push(SearchResult {
                path: path.to_path_buf(),
                file_name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                matches,
            });
        }
    }
    results
}

/// Lists every file under `root` as `(path relative to root, absolute path)`.
///
/// Skips hidden entries, anything gitignored and any directory named in
/// [`crate::features::file_tree::IGNORED_DIRS`], so `node_modules` and the
/// like are not walked even when there is no git repository. This walks the
/// whole workspace, so call it off the UI thread.
pub fn collect_all_files(root: &PathBuf) -> Vec<(String, PathBuf)> {
    use ignore::WalkBuilder;

    let mut files = Vec::new();

    let walker = WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .filter_entry(|e| {
            !crate::features::file_tree::IGNORED_DIRS
                .contains(&e.file_name().to_string_lossy().as_ref())
        })
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let display = path.strip_prefix(root).unwrap_or(path).to_string_lossy().to_string();

        files.push((display, path.to_path_buf()));
    }

    files
}

pub fn fuzzy_find_files(
    query: &str,
    files: &[(String, PathBuf)],
    max_results: usize,
) -> Vec<(i64, String, PathBuf)> {
    let matcher = SkimMatcherV2::default();

    let mut scored: Vec<(i64, String, PathBuf)> = files
        .iter()
        .filter_map(|(display, abs_path)| {
            matcher
                .fuzzy_match(display, query)
                .map(|score| (score, display.clone(), abs_path.clone()))
        })
        .collect();

    scored.sort_by_key(|b| Reverse(b.0));
    scored.truncate(max_results);
    scored
}

#[cfg(test)]
mod tests {
    use std::{error::Error, fs};

    use super::*;

    #[test]
    fn test_collect_all_files_skips_node_modules() -> Result<(), Box<dyn Error>> {
        let dir = tempfile::tempdir()?;
        let root = dir.path().to_path_buf();
        fs::create_dir_all(root.join("src"))?;
        fs::write(root.join("src").join("a.rs"), "fn main() {}")?;
        fs::create_dir_all(root.join("node_modules").join("pkg"))?;
        fs::write(root.join("node_modules").join("pkg").join("index.js"), "")?;

        let files = collect_all_files(&root);

        assert!(files.iter().any(|(_, path)| path == &root.join("src").join("a.rs")));
        assert!(!files.iter().any(|(_, path)| path.starts_with(root.join("node_modules"))));
        Ok(())
    }
}
