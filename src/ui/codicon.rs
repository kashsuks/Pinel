//! UI icons drawn from the [Codicons](https://github.com/microsoft/vscode-codicons)
//! icon font (v0.0.45, CC-BY-4.0).
//!
//! Icons are rendered as text, so they take their colour from the text
//! colour and stay sharp at any size. Codepoints come from the package's
//! `src/template/mapping.json`.

use iced::{
    widget::{text, Text},
    Font,
};

/// Raw bytes of the embedded Codicons font, loaded once at startup.
pub const FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/codicon.ttf");

/// The Codicons font, by the family name stored in the font file.
pub const FONT: Font = Font::with_name("codicon");

/// An icon from the Codicons font.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Codicon {
    /// `files`: the explorer.
    Files,
    /// `source-control`: the git panel.
    SourceControl,
    /// `git-branch`: the current branch.
    GitBranch,
    /// `comment-discussion`: the chat panel.
    CommentDiscussion,
    /// `settings-gear`: settings.
    SettingsGear,
    /// `symbol-array`.
    SymbolArray,
    /// `symbol-boolean`, also used for null.
    SymbolBoolean,
    /// `symbol-class`.
    SymbolClass,
    /// `symbol-constant`.
    SymbolConstant,
    /// `symbol-enum`.
    SymbolEnum,
    /// `symbol-enum-member`.
    SymbolEnumMember,
    /// `symbol-event`.
    SymbolEvent,
    /// `symbol-field`.
    SymbolField,
    /// `symbol-file`.
    SymbolFile,
    /// `symbol-interface`.
    SymbolInterface,
    /// `symbol-key`.
    SymbolKey,
    /// `symbol-method`, also used for functions and constructors.
    SymbolMethod,
    /// `symbol-misc`: kinds with no dedicated icon.
    SymbolMisc,
    /// `symbol-namespace`, also used for modules, packages and objects.
    SymbolNamespace,
    /// `symbol-numeric`.
    SymbolNumeric,
    /// `symbol-operator`.
    SymbolOperator,
    /// `symbol-parameter`, also used for type parameters.
    SymbolParameter,
    /// `symbol-property`.
    SymbolProperty,
    /// `symbol-string`.
    SymbolString,
    /// `symbol-structure`: structs.
    SymbolStructure,
    /// `symbol-variable`.
    SymbolVariable,
}

impl Codicon {
    /// Returns the character this icon occupies in the Codicons font.
    pub const fn char(self) -> char {
        match self {
            Self::Files => '\u{eaf0}',
            Self::SourceControl | Self::GitBranch => '\u{ea68}',
            Self::CommentDiscussion => '\u{eac7}',
            Self::SettingsGear => '\u{eb51}',
            Self::SymbolArray => '\u{ea8a}',
            Self::SymbolBoolean => '\u{ea8f}',
            Self::SymbolClass => '\u{eb5b}',
            Self::SymbolConstant => '\u{eb5d}',
            Self::SymbolEnum => '\u{ea95}',
            Self::SymbolEnumMember => '\u{eb5e}',
            Self::SymbolEvent => '\u{ea86}',
            Self::SymbolField => '\u{eb5f}',
            Self::SymbolFile => '\u{eb60}',
            Self::SymbolInterface => '\u{eb61}',
            Self::SymbolKey => '\u{ea93}',
            Self::SymbolMethod => '\u{ea8c}',
            Self::SymbolMisc => '\u{eb63}',
            Self::SymbolNamespace => '\u{ea8b}',
            Self::SymbolNumeric => '\u{ea90}',
            Self::SymbolOperator => '\u{eb64}',
            Self::SymbolParameter => '\u{ea92}',
            Self::SymbolProperty => '\u{eb65}',
            Self::SymbolString => '\u{eb8d}',
            Self::SymbolStructure => '\u{ea91}',
            Self::SymbolVariable => '\u{ea88}',
        }
    }
}

/// Builds a text widget that draws `icon` at `size` pixels.
///
/// Set the colour with `.color()` like any other text.
pub fn icon<'a>(icon: Codicon, size: f32) -> Text<'a> {
    text(icon.char().to_string())
        .font(FONT)
        .size(size)
        .line_height(1.0)
        .shaping(text::Shaping::Basic)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_is_in_private_use_area() {
        for icon in [
            Codicon::Files,
            Codicon::SymbolVariable,
            Codicon::SettingsGear,
        ] {
            assert!(('\u{e000}'..='\u{f8ff}').contains(&icon.char()));
        }
    }

    #[test]
    fn test_font_family_name_matches_embedded_font() {
        // The family name is stored as plain ASCII in the font's name table;
        // `FONT` must match it or iced silently falls back to the default font.
        let needle = b"codicon";
        assert!(FONT_BYTES.windows(needle.len()).any(|window| window == needle));
    }
}
