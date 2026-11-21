//! Helpers for turning a language server's definition reply into an
//! editor position.

use std::path::PathBuf;

/// Converts a `file://` URI from a language server into a local path.
///
/// Percent-escapes such as `%20` are decoded. Returns `None` for other
/// schemes (for example `jar:` or `untitled:`), which can't be opened as files.
pub fn file_uri_to_path(uri: &str) -> Option<PathBuf> {
    let url = url::Url::parse(uri).ok()?;
    if url.scheme() != "file" {
        return None;
    }
    url.to_file_path().ok()
}

/// Converts an LSP column, counted in UTF-16 code units, into a character
/// column on `line`
///
/// A column past the end of the line clamps to the line's length
pub fn utf16_col_to_char_col(line: &str, utf16_col: u32) -> usize {
    let mut units: u32 = 0;
    for (index, ch) in line.chars().enumerate() {
        if units >= utf16_col {
            return index;
        }
        units += ch.len_utf16() as u32;
    }
    line.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[cfg(unix)]
    #[test]
    fn test_file_uri_to_path_plain() -> Result<(), Box<dyn Error>> {
        let path = file_uri_to_path("file:///home/me/src/main.rs").ok_or("expected a path")?;
        assert_eq!(path, PathBuf::from("/home/me/src/main.rs"));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_file_uri_to_path_decodes_escapes() -> Result<(), Box<dyn Error>> {
        let path =
            file_uri_to_path("file:///home/me/my%20project/lib.rs").ok_or("expected a path")?;
        assert_eq!(path, PathBuf::from("/home/me/my project/lib.rs"));
        Ok(())
    }

    #[test]
    fn test_file_uri_to_path_rejects_other_schemes() {
        assert_eq!(file_uri_to_path("untitled:Untitled-1"), None);
        assert_eq!(file_uri_to_path("not a uri"), None);
    }

    #[test]
    fn test_utf16_col_to_char_col_ascii() {
        assert_eq!(utf16_col_to_char_col("fn main() {}", 3), 3);
    }

    #[test]
    fn test_utf16_col_to_char_col_after_surrogate_pair() {
        // "🦀" is one char but two UTF-16 units, so UTF-16 column 3 is char 2.
        assert_eq!(utf16_col_to_char_col("🦀 x", 3), 2);
    }

    #[test]
    fn test_utf16_col_to_char_col_past_end() {
        assert_eq!(utf16_col_to_char_col("ab", 40), 2);
    }
}
