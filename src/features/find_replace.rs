/// Find and replace - In-editor find and replace with case-sensitive toggle,
/// match navigation, replace-one, and replace-all
#[derive(Default)]
pub struct FindReplace {
    pub open: bool,
    pub find_text: String,
    pub replace_text: String,
    pub case_sensitive: bool,
    pub match_count: usize,
    pub current_match: usize,
    /// Byte ranges `(start, end)` into the original text, one per match
    pub matches: Vec<(usize, usize)>,
}

/// Lowercase `text` returning the lowercased string plus, for every byte of
/// it, the `(start, end)` byte rnage of the original character it came from
fn lowercase_with_map(text: &str) -> (String, Vec<(usize, usize)>) {
    let mut lower = String::with_capacity(text.len());
    let mut map: Vec<(usize, usize)> = Vec::with_capacity(text.len());
    for (orig_start, ch) in text.char_indices() {
        let orig_range = (orig_start, orig_start + ch.len_utf8());
        for lc in ch.to_lowercase() {
            lower.push(lc);
            map.resize(lower.len(), orig_range);
        }
    }
    (lower, map)
}

/// Provides implementation types for the Find and Replace modal
///
/// # Arguments
///
/// - `&mut self` (`undefined`) - Provides the attrubutes for `Self`
impl FindReplace {
    pub fn toggle(&mut self) {
        self.open = !self.open;
        if self.open {
            self.match_count = 0;
            self.current_match = 0;
            self.matches.clear();
        }
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    /// Uses string matching to find matches within scope.
    ///
    /// # Arguments
    ///
    /// - `&mut self` (`undefined`) -  Provides the parameters and arguments for `Self`
    /// - `text` (`&str`) - Text to search and find matches over
    ///
    /// # Returns
    ///
    /// - `Vec<(usize, usize)>` - Non-overalapping byte ranges into `text`, one per match
    pub fn find_matches(&mut self, text: &str) -> Vec<(usize, usize)> {
        if self.find_text.is_empty() {
            self.matches.clear();
            self.match_count = 0;
            return Vec::new();
        }

        let found_matches: Vec<(usize, usize)> = if self.case_sensitive {
            text.match_indices(&self.find_text)
                .map(|(start, matched)| (start, start + matched.len()))
                .collect()
        } else {
            let (lower, map) = lowercase_with_map(text);
            let find = self.find_text.to_lowercase();
            lower
                .match_indices(&find)
                .map(|(start, matched)| (map[start].0, map[start + matched.len() - 1].1))
                .collect()
        };

        self.matches = found_matches.clone();
        self.match_count = found_matches.len();
        found_matches
    }

    pub fn go_to_next_match(&mut self) {
        if !self.matches.is_empty() {
            self.current_match = (self.current_match + 1) % self.matches.len();
        }
    }

    pub fn go_to_prev_match(&mut self) {
        if !self.matches.is_empty() {
            if self.current_match == 0 {
                self.current_match = self.matches.len() - 1;
            } else {
                self.current_match -= 1;
            }
        }
    }

    pub fn replace_next(&mut self, text: &mut String) -> bool {
        if self.matches.is_empty() || self.current_match >= self.matches.len() {
            return false;
        }

        let (start, end) = self.matches[self.current_match];
        text.replace_range(start..end, &self.replace_text);

        self.find_matches(text);

        if self.current_match >= self.matches.len() && !self.matches.is_empty() {
            self.current_match = self.matches.len() - 1;
        }

        true
    }

    pub fn replace_all(&mut self, text: &mut String) -> usize {
        if self.find_text.is_empty() {
            return 0;
        }

        let count = self.matches.len();

        for &(start, end) in self.matches.iter().rev() {
            text.replace_range(start..end, &self.replace_text);
        }

        self.matches.clear();
        self.match_count = 0;
        self.current_match = 0;

        count
    }

    pub fn match_status(&self) -> String {
        if self.find_text.is_empty() {
            String::new()
        } else if self.match_count > 0 {
            format!("{} of {}", self.current_match + 1, self.match_count)
        } else {
            "No matches".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fr(find: &str, replace: &str, case_sensitive: bool) -> FindReplace {
        FindReplace {
            find_text: find.to_string(),
            replace_text: replace.to_string(),
            case_sensitive,
            ..FindReplace::default()
        }
    }

    #[test]
    fn test_find_matches_multibyte_char_returns_all_occurrences() {
        let mut f = fr("é", "", false);
        assert_eq!(f.find_matches("café café"), vec![(3, 5), (9, 11)]);
        assert_eq!(f.match_count, 2);
    }

    #[test]
    fn test_find_matches_multibyte_case_sensitive() {
        let mut f = fr("é", "", true);
        assert_eq!(f.find_matches("café café"), vec![(3, 5), (9, 11)]);
    }

    #[test]
    fn test_find_matches_overlapping_pattern_is_non_overlapping() {
        let mut f = fr("aa", "", true);
        assert_eq!(f.find_matches("aaa"), vec![(0, 2)]);
    }

    #[test]
    fn test_find_matches_case_insensitive_expanding_char() {
        // 'İ' is 2 bytes but lowercases to 3 bytes; offsets must map back.
        let mut f = fr("a", "", false);
        assert_eq!(f.find_matches("İa"), vec![(2, 3)]);
    }

    #[test]
    fn test_replace_all_multibyte_text() {
        let mut f = fr("é", "e", false);
        let mut text = "café café".to_string();
        f.find_matches(&text);
        assert_eq!(f.replace_all(&mut text), 2);
        assert_eq!(text, "cafe cafe");
    }

    #[test]
    fn test_replace_next_expanding_char() {
        let mut f = fr("a", "b", false);
        let mut text = "İa".to_string();
        f.find_matches(&text);
        assert!(f.replace_next(&mut text));
        assert_eq!(text, "İb");
    }
}
