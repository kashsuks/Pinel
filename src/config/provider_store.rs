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
            },
            ProviderCredential {
                provider_id: "anthropic".to_string(),
                api_key: "ant-test".to_string(),
            },
        ];

        let json = serde_json::to_string(&credentials).unwrap();
        let restored: Vec<ProviderCredential> = serde_json::from_str(&json).unwrap();

        assert_eq!(restored, credentials);
    }

    #[test]
    fn deserializing_malformed_json_falls_back_to_empty() {
        let result: Result<Vec<ProviderCredential>, _> = serde_json::from_str("{ not valid json}");
        assert!(result.is_err());
    }
}
