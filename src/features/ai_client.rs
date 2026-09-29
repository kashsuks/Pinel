//! Talks to AI providers: lists available models and sends chat requests.
//!
//! Two request/response shapes are implemented, matching
//! [`crate::features::chat_providers::ProviderApi`]: the OpenAI-compatible
//! shape (OpenAI, OpenRouter, Hack Club AI, Kimi) and Anthropic's Messages
//! API. Both are reached through the same two entry points,
//! [`list_models`] and [`send_message`], so callers don't need to branch on
//! provider.

use serde::{Deserialize, Serialize};

use super::chat::{ChatMessage, ChatRole};
use super::chat_providers::{ProviderApi, ProviderInfo};

const ANTHROPIC_VERSION: &str = "2023-06-01";
const REQUEST_TIMEOUT_SECS: u64 = 60;

/// State of an in-flight or completed model list fetch for one provider,
/// cached in `App` so switching back to a provider in the picker doesn't
/// re-fetch every time.
#[derive(Debug, Clone)]
pub enum ModelFetchState {
    Loading,
    Loaded(Vec<String>),
    Error(String),
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()
        .map_err(|e| e.to_string())
}

/// Fetches the list of model ids a provider currently offers for the given
/// API key.
pub async fn list_models(provider: &ProviderInfo, api_key: &str) -> Result<Vec<String>, String> {
    match provider.api {
        ProviderApi::OpenAiCompatible => list_models_openai_compatible(provider, api_key).await,
        ProviderApi::Anthropic => list_models_anthropic(provider, api_key).await,
    }
}

/// Sends the given conversation to a provider/model and returns the
/// assistant's reply text.
pub async fn send_message(
    provider: &ProviderInfo,
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
) -> Result<String, String> {
    match provider.api {
        ProviderApi::OpenAiCompatible => {
            send_message_openai_compatible(provider, api_key, model, messages).await
        },
        ProviderApi::Anthropic => send_message_anthropic(provider, api_key, model, messages).await,
    }
}

async fn response_error(response: reqwest::Response) -> String {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    match extract_error_message(&body) {
        Some(message) => format!("request failed: {status} - {message}"),
        None if body.trim().is_empty() => format!("request failed: {status}"),
        None => format!("request failed: {status} - {}", body.trim()),
    }
}

/// Pulls a human-readable message out of a provider's JSON error body.
/// OpenAI, OpenRouter, and Anthropic all nest it as `error.message`; falls
/// back to the raw body (handled by the caller) for anything else.
fn extract_error_message(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    value.get("error")?.get("message")?.as_str().map(str::to_string)
}

// ---- OpenAI-compatible (OpenAI, OpenRouter, Hack Club AI, Kimi) ----

#[derive(Deserialize)]
struct OpenAiModelsResponse {
    data: Vec<OpenAiModel>,
}

#[derive(Deserialize)]
struct OpenAiModel {
    id: String,
}

async fn list_models_openai_compatible(
    provider: &ProviderInfo,
    api_key: &str,
) -> Result<Vec<String>, String> {
    let client = client()?;
    let url = format!("{}/models", provider.base_url);
    let response = client.get(url).bearer_auth(api_key).send().await.map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(response_error(response).await);
    }

    let parsed: OpenAiModelsResponse = response.json().await.map_err(|e| e.to_string())?;
    let mut ids: Vec<String> = parsed.data.into_iter().map(|m| m.id).collect();
    ids.sort_unstable();
    Ok(ids)
}

#[derive(Serialize)]
struct OpenAiChatRequest<'a> {
    model: &'a str,
    messages: Vec<OpenAiChatMessage>,
}

#[derive(Serialize)]
struct OpenAiChatMessage {
    role: &'static str,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiChoiceMessage,
}

#[derive(Deserialize)]
struct OpenAiChoiceMessage {
    content: String,
}

async fn send_message_openai_compatible(
    provider: &ProviderInfo,
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
) -> Result<String, String> {
    let client = client()?;
    let url = format!("{}/chat/completions", provider.base_url);

    let body = OpenAiChatRequest {
        model,
        messages: messages.iter().map(to_openai_message).collect(),
    };

    let response = client
        .post(url)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(response_error(response).await);
    }

    let mut parsed: OpenAiChatResponse = response.json().await.map_err(|e| e.to_string())?;
    if parsed.choices.is_empty() {
        return Err("provider returned no choices".to_string());
    }
    Ok(parsed.choices.remove(0).message.content)
}

fn to_openai_message(message: &ChatMessage) -> OpenAiChatMessage {
    OpenAiChatMessage {
        role: role_str(message.role),
        content: message.content.clone(),
    }
}

// ---- Anthropic ----

#[derive(Deserialize)]
struct AnthropicModelsResponse {
    data: Vec<AnthropicModel>,
}

#[derive(Deserialize)]
struct AnthropicModel {
    id: String,
}

async fn list_models_anthropic(
    provider: &ProviderInfo,
    api_key: &str,
) -> Result<Vec<String>, String> {
    let client = client()?;
    let url = format!("{}/models", provider.base_url);
    let response = client
        .get(url)
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(response_error(response).await);
    }

    let parsed: AnthropicModelsResponse = response.json().await.map_err(|e| e.to_string())?;
    let mut ids: Vec<String> = parsed.data.into_iter().map(|m| m.id).collect();
    ids.sort_unstable();
    Ok(ids)
}

#[derive(Serialize)]
struct AnthropicChatRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<OpenAiChatMessage>,
}

#[derive(Deserialize)]
struct AnthropicChatResponse {
    content: Vec<AnthropicContentBlock>,
}

#[derive(Deserialize)]
struct AnthropicContentBlock {
    #[serde(default)]
    text: String,
}

const ANTHROPIC_MAX_TOKENS: u32 = 4096;

async fn send_message_anthropic(
    provider: &ProviderInfo,
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
) -> Result<String, String> {
    let client = client()?;
    let url = format!("{}/messages", provider.base_url);

    let body = AnthropicChatRequest {
        model,
        max_tokens: ANTHROPIC_MAX_TOKENS,
        messages: messages.iter().map(to_openai_message).collect(),
    };

    let response = client
        .post(url)
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(response_error(response).await);
    }

    let parsed: AnthropicChatResponse = response.json().await.map_err(|e| e.to_string())?;
    let text = parsed.content.into_iter().map(|b| b.text).collect::<Vec<_>>().join("");
    if text.is_empty() {
        return Err("provider returned no content".to_string());
    }
    Ok(text)
}

fn role_str(role: ChatRole) -> &'static str {
    match role {
        ChatRole::User => "user",
        ChatRole::Assistant => "assistant",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_message_from_openrouter_style_error() {
        let body = r#"{"error":{"message":"Provider returned error","code":429,"metadata":{"raw":"qwen/qwen3.8-27b:free is temporarily rate-limited upstream.","provider_name":"ModelRun"}},"user_id":"user_abc"}"#;
        assert_eq!(
            extract_error_message(body),
            Some("Provider returned error".to_string())
        );
    }

    #[test]
    fn extracts_message_from_anthropic_style_error() {
        let body = r#"{"type":"error","error":{"type":"invalid_request_error","message":"model: field required"}}"#;
        assert_eq!(
            extract_error_message(body),
            Some("model: field required".to_string())
        );
    }

    #[test]
    fn falls_back_to_none_for_non_json_body() {
        assert_eq!(extract_error_message("Internal Server Error"), None);
    }

    #[test]
    fn falls_back_to_none_when_error_field_missing() {
        assert_eq!(extract_error_message(r#"{"message":"oops"}"#), None);
    }
}
