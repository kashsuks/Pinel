//! Minimal LSP types and helpers used by the editor.

pub(crate) mod sync;

#[cfg(all(feature = "lsp-process", not(target_arch = "wasm32")))]
pub mod process;

/// A zero-based position in an LSP document.
///
/// Both fields are zero-based, unlike the one-based line numbers shown in the
/// editor's gutter.
///
/// # Example
///
/// ```
/// use iced_code_editor::LspPosition;
///
/// // The gutter's "line 3, column 1" is this position.
/// let position = LspPosition { line: 2, character: 0 };
/// assert_eq!(position.line, 2);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspPosition {
    /// Zero-based line index.
    pub line: u32,
    /// Zero-based character index on the line.
    pub character: u32,
}

/// Metadata describing the currently edited document.
///
/// The editor stamps `version` itself: [`CodeEditor::attach_lsp`] sets it to 1
/// on open, and each flushed batch of changes increments it.
///
/// # Example
///
/// ```
/// use iced_code_editor::LspDocument;
///
/// let document = LspDocument::new("file:///tmp/main.rs", "rust");
/// assert_eq!(document.uri, "file:///tmp/main.rs");
/// assert_eq!(document.language_id, "rust");
/// ```
///
/// [`CodeEditor::attach_lsp`]: crate::CodeEditor::attach_lsp
#[derive(Debug, Clone)]
pub struct LspDocument {
    /// Document URI.
    pub uri: String,
    /// Language identifier for syntax services.
    pub language_id: String,
    /// Version number used for LSP change notifications.
    pub version: i32,
}

impl LspDocument {
    /// Creates a new LSP document descriptor with version set to 0.
    ///
    /// # Arguments
    ///
    /// * `uri` - The document URI, e.g. `file:///path/to/main.rs`
    /// * `language_id` - The LSP language identifier, e.g. `rust`
    ///
    /// # Returns
    ///
    /// A descriptor at version 0; the editor stamps version 1 on open
    ///
    /// # Example
    ///
    /// ```
    /// use iced_code_editor::LspDocument;
    ///
    /// let document = LspDocument::new("file:///tmp/main.rs", "rust");
    /// assert_eq!(document.version, 0);
    /// ```
    pub fn new(uri: impl Into<String>, language_id: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            language_id: language_id.into(),
            version: 0,
        }
    }
}

/// A text range in an LSP document.
///
/// `start` is inclusive and `end` is exclusive, so an empty range (a pure
/// insertion point) has `start == end`.
///
/// # Example
///
/// ```
/// use iced_code_editor::{LspPosition, LspRange};
///
/// // The first four characters of the first line.
/// let range = LspRange {
///     start: LspPosition { line: 0, character: 0 },
///     end: LspPosition { line: 0, character: 4 },
/// };
/// assert_eq!(range.end.character - range.start.character, 4);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct LspRange {
    /// Range start (inclusive).
    pub start: LspPosition,
    /// Range end (exclusive).
    pub end: LspPosition,
}

/// A text change described by a range replacement.
///
/// An insertion is an empty `range` with non-empty `text`; a deletion is a
/// non-empty `range` with empty `text`.
///
/// # Example
///
/// ```
/// use iced_code_editor::{LspPosition, LspRange, LspTextChange};
///
/// // Insert "hi" at the start of the document: an empty range.
/// let insertion = LspTextChange {
///     range: LspRange {
///         start: LspPosition { line: 0, character: 0 },
///         end: LspPosition { line: 0, character: 0 },
///     },
///     text: "hi".to_string(),
/// };
/// assert_eq!(insertion.range.start, insertion.range.end);
/// ```
#[derive(Debug, Clone)]
pub struct LspTextChange {
    /// Range replaced by the change.
    pub range: LspRange,
    /// Inserted text.
    pub text: String,
}

/// The kind of an LSP document symbol.
///
/// The variants correspond to the numeric values in the Language Server
/// Protocol's `SymbolKind` type. [`Self::Unknown`] keeps responses from older
/// or non-conforming servers usable when their kind is missing or unsupported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspSymbolKind {
    /// A file.
    File,
    /// A module.
    Module,
    /// A namespace.
    Namespace,
    /// A package.
    Package,
    /// A class.
    Class,
    /// A method.
    Method,
    /// A property.
    Property,
    /// A field.
    Field,
    /// A constructor.
    Constructor,
    /// An enum.
    Enum,
    /// An interface.
    Interface,
    /// A function.
    Function,
    /// A variable.
    Variable,
    /// A constant.
    Constant,
    /// A string value.
    String,
    /// A numeric value.
    Number,
    /// A boolean value.
    Boolean,
    /// An array.
    Array,
    /// An object.
    Object,
    /// An object key.
    Key,
    /// A null value.
    Null,
    /// An enum member.
    EnumMember,
    /// A struct.
    Struct,
    /// An event.
    Event,
    /// An operator.
    Operator,
    /// A type parameter.
    TypeParameter,
    /// A missing or unsupported symbol kind.
    Unknown,
}

impl LspSymbolKind {
    /// Converts the numeric value used by the Language Server Protocol.
    pub fn from_lsp_value(value: Option<u64>) -> Self {
        match value {
            Some(1) => Self::File,
            Some(2) => Self::Module,
            Some(3) => Self::Namespace,
            Some(4) => Self::Package,
            Some(5) => Self::Class,
            Some(6) => Self::Method,
            Some(7) => Self::Property,
            Some(8) => Self::Field,
            Some(9) => Self::Constructor,
            Some(10) => Self::Enum,
            Some(11) => Self::Interface,
            Some(12) => Self::Function,
            Some(13) => Self::Variable,
            Some(14) => Self::Constant,
            Some(15) => Self::String,
            Some(16) => Self::Number,
            Some(17) => Self::Boolean,
            Some(18) => Self::Array,
            Some(19) => Self::Object,
            Some(20) => Self::Key,
            Some(21) => Self::Null,
            Some(22) => Self::EnumMember,
            Some(23) => Self::Struct,
            Some(24) => Self::Event,
            Some(25) => Self::Operator,
            Some(26) => Self::TypeParameter,
            _ => Self::Unknown,
        }
    }
}

/// A symbol in a document's outline, as reported by `textDocument/documentSymbol`.
///
/// Symbols form a tree: a method sits in the `children` of its type, which
/// sits in the `children` of its module. Servers that only return a flat list
/// are nested by range containment when the response is parsed.
///
/// # Example
///
/// ```
/// use iced_code_editor::{LspPosition, LspRange, LspSymbol, LspSymbolKind};
///
/// let symbol = LspSymbol {
///     name: "main".to_string(),
///     kind: LspSymbolKind::Function,
///     range: LspRange {
///         start: LspPosition { line: 0, character: 0 },
///         end: LspPosition { line: 2, character: 1 },
///     },
///     children: Vec::new(),
/// };
/// assert!(symbol.contains_line(1));
/// assert!(!symbol.contains_line(3));
/// ```
#[derive(Debug, Clone)]
pub struct LspSymbol {
    /// Name shown for the symbol, such as a function or type name.
    pub name: String,
    /// Semantic kind reported by the language server.
    pub kind: LspSymbolKind,
    /// Full range of the symbol, including its body.
    pub range: LspRange,
    /// Symbols declared inside this one.
    pub children: Vec<LspSymbol>,
}

impl LspSymbol {
    /// Returns `true` if the zero-based `line` falls inside this symbol's range.
    ///
    /// # Arguments
    ///
    /// * `line` - Zero-based line number to test
    ///
    /// # Returns
    ///
    /// `true` when `line` is between the range's start and end lines, inclusive
    pub fn contains_line(&self, line: u32) -> bool {
        self.range.start.line <= line && line <= self.range.end.line
    }
}

/// Returns the names of the symbols enclosing `line`, outermost first.
///
/// At each level the first symbol whose range contains `line` is entered, so
/// the result reads like a breadcrumb trail: `["Editor", "render"]` for a
/// line inside the `render` method of `Editor`.
///
/// # Arguments
///
/// * `symbols` - Top-level symbols of the document
/// * `line` - Zero-based line number
///
/// # Returns
///
/// The chain of enclosing symbol names, empty when `line` is outside every symbol
///
/// # Example
///
/// ```
/// use iced_code_editor::{LspPosition, LspRange, LspSymbol, LspSymbolKind, symbol_path_at};
///
/// let range = |start, end| LspRange {
///     start: LspPosition { line: start, character: 0 },
///     end: LspPosition { line: end, character: 0 },
/// };
/// let symbols = vec![LspSymbol {
///     name: "Editor".to_string(),
///     kind: LspSymbolKind::Struct,
///     range: range(0, 10),
///     children: vec![LspSymbol {
///         name: "render".to_string(),
///         kind: LspSymbolKind::Method,
///         range: range(2, 5),
///         children: Vec::new(),
///     }],
/// }];
///
/// assert_eq!(symbol_path_at(&symbols, 3), ["Editor", "render"]);
/// assert_eq!(symbol_path_at(&symbols, 8), ["Editor"]);
/// assert!(symbol_path_at(&symbols, 11).is_empty());
/// ```
pub fn symbol_path_at(symbols: &[LspSymbol], line: u32) -> Vec<String> {
    symbol_details_path_at(symbols, line)
        .into_iter()
        .map(|symbol| symbol.name.clone())
        .collect()
}

/// Returns the symbols enclosing `line`, outermost first.
///
/// Unlike [`symbol_path_at`], this keeps each symbol's kind and range
/// available to callers that need richer breadcrumb presentation.
pub fn symbol_details_path_at(symbols: &[LspSymbol], line: u32) -> Vec<&LspSymbol> {
    let mut path = Vec::new();
    let mut level = symbols;
    while let Some(symbol) = level.iter().find(|symbol| symbol.contains_line(line)) {
        path.push(symbol);
        level = &symbol.children;
    }
    path
}

/// LSP client hooks invoked by the editor.
///
/// Every method has a no-op default, so an implementation only overrides the
/// notifications and requests it actually cares about. Attach one with
/// [`CodeEditor::attach_lsp`]; the editor never blocks on a response, so
/// request results come back through whatever channel the implementation
/// chooses.
///
/// # Example
///
/// ```
/// use std::cell::RefCell;
/// use std::rc::Rc;
///
/// use iced_code_editor::{CodeEditor, LspClient, LspDocument, LspTextChange};
///
/// /// A client that only records how many change batches it received.
/// struct CountingClient(Rc<RefCell<usize>>);
///
/// impl LspClient for CountingClient {
///     fn did_change(&mut self, _document: &LspDocument, changes: &[LspTextChange]) {
///         *self.0.borrow_mut() += changes.len();
///     }
/// }
///
/// let changes = Rc::new(RefCell::new(0));
/// let mut editor = CodeEditor::new("fn main() {}", "rs");
/// editor.attach_lsp(
///     Box::new(CountingClient(Rc::clone(&changes))),
///     LspDocument::new("file:///tmp/main.rs", "rust"),
/// );
///
/// // `did_open` fired, but no change has been flushed yet.
/// assert_eq!(*changes.borrow(), 0);
/// ```
///
/// [`CodeEditor::attach_lsp`]: crate::CodeEditor::attach_lsp
pub trait LspClient {
    /// Notifies the client that a document was opened.
    fn did_open(&mut self, _document: &LspDocument, _text: &str) {}
    /// Notifies the client that the document changed.
    fn did_change(&mut self, _document: &LspDocument, _changes: &[LspTextChange]) {}
    /// Notifies the client that the document was saved.
    fn did_save(&mut self, _document: &LspDocument, _text: &str) {}
    /// Notifies the client that the document was closed.
    fn did_close(&mut self, _document: &LspDocument) {}
    /// Requests hover information at the given position.
    fn request_hover(&mut self, _document: &LspDocument, _position: LspPosition) {}
    /// Requests completion items at the given position.
    fn request_completion(&mut self, _document: &LspDocument, _position: LspPosition) {}
    /// Requests the definition location(s) for the symbol at the given position.
    ///
    /// This method is called when the user triggers a "Go to Definition" action
    /// (e.g., via Ctrl+Click or a context menu). The client implementation should
    /// send a `textDocument/definition` request to the LSP server.
    fn request_definition(&mut self, _document: &LspDocument, _position: LspPosition) {}
    /// Requests the outline (`textDocument/documentSymbol`) of the document.
    ///
    /// The client implementation should send the request and report the
    /// symbols back through its own channel, as with the other requests.
    fn request_document_symbols(&mut self, _document: &LspDocument) {}
}

/// Computes a minimal text change between two snapshots.
///
/// The change is narrowed by trimming the common prefix and suffix, so a
/// one-character edit in a large document produces a one-character change
/// rather than a whole-document replacement. This is what lets the editor send
/// incremental `didChange` notifications instead of resending the buffer.
///
/// # Arguments
///
/// * `old` - The previous document contents
/// * `new` - The current document contents
///
/// # Returns
///
/// `Some(change)` describing the edit, or `None` when the snapshots are identical
///
/// # Example
///
/// ```
/// use iced_code_editor::compute_text_change;
///
/// // Identical snapshots produce no change at all.
/// assert!(compute_text_change("fn main() {}", "fn main() {}").is_none());
///
/// // Only the differing region is reported, not the whole line.
/// let change = compute_text_change("let x = 1;", "let x = 42;")
///     .expect("the snapshots differ");
/// assert_eq!(change.text, "42");
/// assert_eq!(change.range.start.character, 8);
/// assert_eq!(change.range.end.character, 9);
/// ```
pub fn compute_text_change(old: &str, new: &str) -> Option<LspTextChange> {
    if old == new {
        return None;
    }

    let old_chars: Vec<char> = old.chars().collect();
    let new_chars: Vec<char> = new.chars().collect();
    let old_len = old_chars.len();
    let new_len = new_chars.len();

    let mut prefix = 0;
    while prefix < old_len && prefix < new_len && old_chars[prefix] == new_chars[prefix] {
        prefix += 1;
    }

    let mut suffix = 0;
    while suffix < old_len.saturating_sub(prefix)
        && suffix < new_len.saturating_sub(prefix)
        && old_chars[old_len - 1 - suffix] == new_chars[new_len - 1 - suffix]
    {
        suffix += 1;
    }

    let removed_len = old_len.saturating_sub(prefix + suffix);
    let inserted: String = new_chars[prefix..new_len.saturating_sub(suffix)].iter().collect();

    let start = position_for_char_index(old, prefix);
    let end = position_for_char_index(old, prefix + removed_len);

    Some(LspTextChange {
        range: LspRange { start, end },
        text: inserted,
    })
}

/// Converts a character index into a line/character position.
fn position_for_char_index(text: &str, target_index: usize) -> LspPosition {
    let mut line: u32 = 0;
    let mut character: u32 = 0;
    for (index, ch) in text.chars().enumerate() {
        if index == target_index {
            return LspPosition { line, character };
        }
        if ch == '\n' {
            line = line.saturating_add(1);
            character = 0;
        } else {
            character = character.saturating_add(1);
        }
    }

    LspPosition { line, character }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn symbol(name: &str, start: u32, end: u32, children: Vec<LspSymbol>) -> LspSymbol {
        LspSymbol {
            name: name.to_string(),
            kind: LspSymbolKind::Unknown,
            range: LspRange {
                start: LspPosition {
                    line: start,
                    character: 0,
                },
                end: LspPosition {
                    line: end,
                    character: 0,
                },
            },
            children,
        }
    }

    #[test]
    fn test_symbol_path_at_picks_matching_sibling() {
        let symbols = vec![
            symbol("first", 0, 3, Vec::new()),
            symbol("second", 5, 9, vec![symbol("inner", 6, 7, Vec::new())]),
        ];
        assert_eq!(symbol_path_at(&symbols, 6), ["second", "inner"]);
        assert_eq!(symbol_path_at(&symbols, 9), ["second"]);
        assert_eq!(symbol_path_at(&symbols, 2), ["first"]);
    }

    #[test]
    fn test_symbol_path_at_between_symbols_is_empty() {
        let symbols = vec![
            symbol("first", 0, 3, Vec::new()),
            symbol("second", 5, 9, Vec::new()),
        ];
        assert!(symbol_path_at(&symbols, 4).is_empty());
        assert!(symbol_path_at(&[], 0).is_empty());
    }

    #[test]
    fn test_symbol_details_path_at_preserves_symbol_kinds() {
        let mut outer = symbol("Editor", 0, 9, vec![symbol("render", 2, 5, Vec::new())]);
        outer.kind = LspSymbolKind::Class;
        outer.children[0].kind = LspSymbolKind::Method;

        let symbols = [outer];
        let path = symbol_details_path_at(&symbols, 3);
        assert_eq!(path.len(), 2);
        assert_eq!(path[0].kind, LspSymbolKind::Class);
        assert_eq!(path[1].kind, LspSymbolKind::Method);
    }

    #[test]
    fn test_compute_text_change_none_when_equal() {
        let change = compute_text_change("abc", "abc");
        assert!(change.is_none());
    }

    #[test]
    fn test_compute_text_change_insertion() {
        let change = compute_text_change("abc", "abXc");
        assert!(change.is_some());
        if let Some(change) = change {
            assert_eq!(change.text, "X");
            assert_eq!(
                change.range.start,
                LspPosition {
                    line: 0,
                    character: 2
                }
            );
            assert_eq!(
                change.range.end,
                LspPosition {
                    line: 0,
                    character: 2
                }
            );
        }
    }

    #[test]
    fn test_compute_text_change_deletion_across_lines() {
        let change = compute_text_change("a\nbc", "a\nc");
        assert!(change.is_some());
        if let Some(change) = change {
            assert_eq!(change.text, "");
            assert_eq!(
                change.range.start,
                LspPosition {
                    line: 1,
                    character: 0
                }
            );
            assert_eq!(
                change.range.end,
                LspPosition {
                    line: 1,
                    character: 1
                }
            );
        }
    }

    #[test]
    fn test_position_for_char_index_end_of_text() {
        let pos = position_for_char_index("a\nb", 3);
        assert_eq!(
            pos,
            LspPosition {
                line: 1,
                character: 1
            }
        );
    }
}
