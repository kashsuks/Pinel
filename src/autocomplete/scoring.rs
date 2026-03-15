use crate::autocomplete::context::CompletionContext;

pub struct FuzzyScorer;

impl FuzzyScorer {
    pub fn score(text: &str, pattern: &str) -> f32 {
        if pattern.is_empty() {
            return 0.0;
        }

        let text_lower = text.to_lowercase();
        let pattern_lower = pattern.to_lowercase();

        if text_lower == pattern_lower {
            return 1000.0;
        }

        // Exact prefix match — strongly preferred, shorter candidates rank higher
        if text_lower.starts_with(&pattern_lower) {
            let length_penalty = (text.len() - pattern.len()) as f32;
            // Penalize leading underscores (e.g. __import__ vs import)
            let underscore_penalty = text.chars().take_while(|c| *c == '_').count() as f32 * 50.0;
            return 900.0 - length_penalty - underscore_penalty;
        }

        Self::fuzzy_match_score(text, pattern, &text_lower, &pattern_lower)
    }

    fn fuzzy_match_score(text: &str, _pattern: &str, text_lower: &str, pattern_lower: &str) -> f32 {
        let mut score = 0.0;
        let mut pattern_idx = 0;
        let text_chars: Vec<char> = text_lower.chars().collect();
        let pattern_chars: Vec<char> = pattern_lower.chars().collect();
        let text_original: Vec<char> = text.chars().collect();

        let mut last_match_idx = None;

        for (i, &ch) in text_chars.iter().enumerate() {
            if pattern_idx < pattern_chars.len() && ch == pattern_chars[pattern_idx] {
                score += 100.0;

                if let Some(last_idx) = last_match_idx {
                    if i == last_idx + 1 {
                        score += 50.0;
                    }
                }

                if i == 0 || !text_chars[i - 1].is_alphanumeric() {
                    score += 30.0;
                }

                if i < text_original.len() && text_original[i].is_uppercase() {
                    score += 20.0;
                }

                last_match_idx = Some(i);
                pattern_idx += 1;
            }
        }

        if pattern_idx == pattern_chars.len() {
            score -= (text_chars.len() - pattern_chars.len()) as f32 * 2.0;
            score
        } else {
            0.0
        }
    }

    pub fn apply_context_boost(
        score: f32,
        kind: &crate::autocomplete::types::SuggestionKind,
        context: &CompletionContext,
    ) -> f32 {
        let mut adjusted_score = score;

        if context.is_type_position
            && matches!(kind, crate::autocomplete::types::SuggestionKind::Type)
        {
            adjusted_score += 200.0;
        }

        if context.is_member_access
            && matches!(
                kind,
                crate::autocomplete::types::SuggestionKind::Method
                    | crate::autocomplete::types::SuggestionKind::Property
            )
        {
            adjusted_score += 150.0;
        }

        adjusted_score
    }

    pub fn apply_recency_boost(score: f32, is_recent: bool) -> f32 {
        if is_recent {
            score + 100.0
        } else {
            score
        }
    }
}
