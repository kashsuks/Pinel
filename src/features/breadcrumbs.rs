//! Breadcrumb trail shown under the tab bar: `project › dir › file › symbol`.

use std::path::Path;

use iced_code_editor::LspSymbolKind;

/// Returns a compact glyph for an LSP symbol kind in the breadcrumb bar.
pub(crate) const fn symbol_icon(kind: LspSymbolKind) -> &'static str {
    match kind {
        LspSymbolKind::File => "▤",
        LspSymbolKind::Module => "⬡",
        LspSymbolKind::Namespace => "N",
        LspSymbolKind::Package => "▣",
        LspSymbolKind::Class => "C",
        LspSymbolKind::Method => "m",
        LspSymbolKind::Property => "◈",
        LspSymbolKind::Field => "▪",
        LspSymbolKind::Constructor => "◇",
        LspSymbolKind::Enum => "E",
        LspSymbolKind::Interface => "I",
        LspSymbolKind::Function => "ƒ",
        LspSymbolKind::Variable => "𝑥",
        LspSymbolKind::Constant => "π",
        LspSymbolKind::String => "S",
        LspSymbolKind::Number => "#",
        LspSymbolKind::Boolean => "◉",
        LspSymbolKind::Array => "[]",
        LspSymbolKind::Object => "⬡",
        LspSymbolKind::Key => "◆",
        LspSymbolKind::Null => "∅",
        LspSymbolKind::EnumMember => "•",
        LspSymbolKind::Struct => "S",
        LspSymbolKind::Event => "⚡",
        LspSymbolKind::Operator => "±",
        LspSymbolKind::TypeParameter => "T",
        LspSymbolKind::Unknown => "◇",
    }
}

/// Returns the path part of the breadcrumb for `path`.
///
/// When `path` is inside `root`, the trail starts with the project folder
/// name followed by each component relative to it. Otherwise only the file
/// name is returned, since an absolute path would be too long to be useful.
///
/// # Arguments
///
/// * `path` - Path of the open file
/// * `root` - Folder currently open in the file tree, if any
///
/// # Returns
///
/// The breadcrumb segments, outermost first
///
/// # Examples
///
/// ```ignore
/// use std::path::Path;
///
/// let crumbs = path_segments(Path::new("/w/pinel/src/app.rs"), Some(Path::new("/w/pinel")));
/// assert_eq!(crumbs, ["pinel", "src", "app.rs"]);
/// ```
pub fn path_segments(path: &Path, root: Option<&Path>) -> Vec<String> {
    if let Some(root) = root {
        if let Ok(relative) = path.strip_prefix(root) {
            let project = root.file_name().map_or_else(
                || root.display().to_string(),
                |name| name.to_string_lossy().into_owned(),
            );
            return std::iter::once(project)
                .chain(relative.iter().map(|part| part.to_string_lossy().into_owned()))
                .collect();
        }
    }

    path.file_name()
        .map(|name| vec![name.to_string_lossy().into_owned()])
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_segments_inside_root_starts_with_project_name() {
        let crumbs = path_segments(
            Path::new("/work/pinel/src/app/update.rs"),
            Some(Path::new("/work/pinel")),
        );
        assert_eq!(crumbs, ["pinel", "src", "app", "update.rs"]);
    }

    #[test]
    fn test_path_segments_outside_root_is_file_name_only() {
        let crumbs = path_segments(Path::new("/etc/hosts"), Some(Path::new("/work/pinel")));
        assert_eq!(crumbs, ["hosts"]);
    }

    #[test]
    fn test_path_segments_without_root_is_file_name_only() {
        assert_eq!(
            path_segments(Path::new("/work/notes.md"), None),
            ["notes.md"]
        );
    }

    #[test]
    fn test_path_segments_filesystem_root_uses_display_name() {
        let crumbs = path_segments(Path::new("/notes.md"), Some(Path::new("/")));
        assert_eq!(crumbs, ["/", "notes.md"]);
    }

    #[test]
    fn test_symbol_icon_distinguishes_common_symbol_kinds() {
        assert_eq!(symbol_icon(LspSymbolKind::Function), "ƒ");
        assert_eq!(symbol_icon(LspSymbolKind::Class), "C");
        assert_eq!(symbol_icon(LspSymbolKind::Object), "⬡");
        assert_eq!(symbol_icon(LspSymbolKind::Method), "m");
    }
}
