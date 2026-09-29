//! Data model for the AI chat tab.
//!
//! This models the conversation shape and local bookkeeping. Talking to a
//! provider happens in `features::ai_client`; this module just holds the
//! resulting messages.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Who a given message in a chat session came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatRole {
    User,
    Assistant,
}

/// A single message within a [`ChatSession`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
    /// Unix timestamp, in seconds, of when the message was sent.
    pub timestamp: u64,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
            timestamp: now_unix(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content.into(),
            timestamp: now_unix(),
        }
    }
}

/// One conversation with a chosen provider/model, plus its message history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub provider: String,
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl ChatSession {
    /// Creates a new, empty chat session for the given provider/model.
    pub fn new(provider: impl Into<String>, model: impl Into<String>) -> Self {
        let now = now_unix();
        Self {
            id: generate_id(),
            title: "New Chat".to_string(),
            provider: provider.into(),
            model: model.into(),
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Appends a message to the session and refreshes `updated_at`.
    pub fn push(&mut self, message: ChatMessage) {
        self.updated_at = message.timestamp;
        self.messages.push(message);
    }
}

fn now_unix() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// Generates a locally-unique session id from the current time plus a
/// monotonic counter, so two sessions created within the same clock tick
/// (clock resolution varies by platform) never collide, without pulling in
/// a UUID dependency.
fn generate_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("chat-{nanos}-{seq}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_session_starts_empty_with_default_title() {
        let session = ChatSession::new("OpenAI", "gpt-4o");
        assert_eq!(session.title, "New Chat");
        assert_eq!(session.provider, "OpenAI");
        assert_eq!(session.model, "gpt-4o");
        assert!(session.messages.is_empty());
        assert_eq!(session.created_at, session.updated_at);
    }

    #[test]
    fn pushing_a_message_updates_the_session() {
        let mut session = ChatSession::new("OpenAI", "gpt-4o");
        session.push(ChatMessage::user("hello"));

        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].content, "hello");
        assert_eq!(session.messages[0].role, ChatRole::User);
    }

    #[test]
    fn session_ids_are_unique() {
        let a = ChatSession::new("OpenAI", "gpt-4o");
        let b = ChatSession::new("OpenAI", "gpt-4o");
        assert_ne!(a.id, b.id);
    }
}
