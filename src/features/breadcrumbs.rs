//! Breadcrumb trail shown under the tab bar: `project › dir › file › symbol`.

use std::path::Path;

use iced_code_editor::LspSymbolKind;

use crate::ui::codicon::Codicon;

/// Returns the Codicon for an LSP symbol kind in the breadcrumb bar.
///
/// Kinds without a dedicated icon share the one VS Code uses for them.
pub(crate) const fn symbol_icon(kind: LspSymbolKind) -> Codicon {
    match kind {
        LspSymbolKind::File => Codicon::SymbolFile,
        LspSymbolKind::Module
        | LspSymbolKind::Namespace
        | LspSymbolKind::Package
        | LspSymbolKind::Object => Codicon::SymbolNamespace,
        LspSymbolKind::Class => Codicon::SymbolClass,
        LspSymbolKind::Method | LspSymbolKind::Function | LspSymbolKind::Constructor => {
            Codicon::SymbolMethod
        },
        LspSymbolKind::Property => Codicon::SymbolProperty,
        LspSymbolKind::Field => Codicon::SymbolField,
        LspSymbolKind::Enum => Codicon::SymbolEnum,
        LspSymbolKind::Interface => Codicon::SymbolInterface,
        LspSymbolKind::Variable => Codicon::SymbolVariable,
        LspSymbolKind::Constant => Codicon::SymbolConstant,
        LspSymbolKind::String => Codicon::SymbolString,
        LspSymbolKind::Number => Codicon::SymbolNumeric,
        LspSymbolKind::Boolean | LspSymbolKind::Null => Codicon::SymbolBoolean,
        LspSymbolKind::Array => Codicon::SymbolArray,
        LspSymbolKind::Key => Codicon::SymbolKey,
        LspSymbolKind::EnumMember => Codicon::SymbolEnumMember,
        LspSymbolKind::Struct => Codicon::SymbolStructure,
        LspSymbolKind::Event => Codicon::SymbolEvent,
        LspSymbolKind::Operator => Codicon::SymbolOperator,
        LspSymbolKind::TypeParameter => Codicon::SymbolParameter,
        LspSymbolKind::Unknown => Codicon::SymbolMisc,
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
        assert_eq!(symbol_icon(LspSymbolKind::Function), Codicon::SymbolMethod);
        assert_eq!(symbol_icon(LspSymbolKind::Class), Codicon::SymbolClass);
        assert_eq!(symbol_icon(LspSymbolKind::Object), Codicon::SymbolNamespace);
        assert_eq!(symbol_icon(LspSymbolKind::Struct), Codicon::SymbolStructure);
        assert_eq!(symbol_icon(LspSymbolKind::String), Codicon::SymbolString);
    }

    #[test]
    fn test_symbol_icon_struct_and_string_differ() {
        // These both used to render as "S".
        assert_ne!(
            symbol_icon(LspSymbolKind::Struct),
            symbol_icon(LspSymbolKind::String)
        );
    }
}
