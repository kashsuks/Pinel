//! Persists AI provider API keys.
//!
//! Unlike `chat_store.rs`, this is not scoped per workspace - a provider's
//! key is a machine-wide credential. Stored as plaintext JSON under
//! `~/.config/pinel/`, consistent with how `wakatime/config.rs` keeps its
//! API key (no OS keychain integration yet).

use super::theme_manager::get_config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderCredential {
    pub provider_id: String,
    pub api_key: String,
    /// Overrides the provider's default base URL. Used for locally-hosted
    /// providers (Ollama, LM Studio, ...) whose host/port depends on how
    /// the user has set up their local server. `#[serde(default)]` keeps
    /// older `providers.json` files (saved before this field existed)
    /// loading correctly.
    #[serde(default)]
    pub base_url: Option<String>,
}

fn credentials_path() -> PathBuf {
    get_config_dir().join("providers.json")
}

/// Loads all saved provider credentials. Returns an empty list on first run
/// or if the file is missing/corrupted.
pub fn load() -> Vec<ProviderCredential> {
    let Ok(content) = fs::read_to_string(credentials_path()) else {
        return Vec::new();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

/// Persists all provider credentials, overwriting whatever was previously
/// saved.
pub fn save(credentials: &[ProviderCredential]) -> std::io::Result<()> {
    let path = credentials_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(credentials).unwrap_or_else(|_| "[]".to_string());
    fs::write(path, json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_list_serialize_and_deserialize_roundtrip() {
        let credentials = vec![
            ProviderCredential {
                provider_id: "openai".to_string(),
                api_key: "sk-test".to_string(),
                base_url: None,
            },
            ProviderCredential {
                provider_id: "ollama".to_string(),
                api_key: String::new(),
                base_url: Some("http://192.168.1.50:11434/v1".to_string()),
            },
        ];

        let json = serde_json::to_string(&credentials).unwrap();
        let restored: Vec<ProviderCredential> = serde_json::from_str(&json).unwrap();

        assert_eq!(restored, credentials);
    }

    #[test]
    fn deserializing_credentials_saved_before_base_url_existed_still_works() {
        let legacy_json = r#"[{"provider_id":"openai","api_key":"sk-test"}]"#;
        let restored: Vec<ProviderCredential> = serde_json::from_str(legacy_json).unwrap();

        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].provider_id, "openai");
        assert_eq!(restored[0].base_url, None);
    }

    #[test]
    fn deserializing_malformed_json_falls_back_to_empty() {
        let result: Result<Vec<ProviderCredential>, _> = serde_json::from_str("{ not valid json}");
        assert!(result.is_err());
    }
}
