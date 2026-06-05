#![allow(dead_code)]

#[derive(Default, Debug, Clone)]
pub struct CompletionContext {
    pub is_member_access: bool,
    pub is_function_call: bool,
    pub is_namespace_access: bool,
    pub is_type_position: bool,
}

impl CompletionContext {
    pub fn analyze(text: &str, cursor_pos: usize) -> Self {
        let mut context = Self::default();
        let cursor_pos = cursor_pos.min(text.len());

        if cursor_pos == 0 {
            return context;
        }

        let before_cursor = &text[..cursor_pos];
        let after_cursor = &text[cursor_pos..];

        if before_cursor.ends_with('.') || before_cursor.trim_end().ends_with('.') {
            context.is_member_access = true;
        }

        if after_cursor.trim_start().starts_with('(') {
            context.is_function_call = true;
        }

        if before_cursor.ends_with("::") {
            context.is_namespace_access = true;
        }

        if before_cursor.trim_end().ends_with(':') || before_cursor.trim_end().ends_with("->") {
            context.is_type_position = true;
        }

        context
    }

    pub fn should_show_keywords(&self) -> bool {
        !self.is_member_access && !self.is_namespace_access
    }

    pub fn should_boost_types(&self) -> bool {
        self.is_type_position
    }

    pub fn should_show_member(&self) -> bool {
        self.is_member_access || self.is_namespace_access
    }
}
