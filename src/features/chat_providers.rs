//! Catalog of AI providers the chat tab can talk to.
//!
//! Each entry carries what's needed to actually call that provider: its API
//! base URL and which request/response shape it speaks. Model lists are not
//! stored here - they're fetched live from each provider once a key is
//! attached (see `features::ai_client`).

/// Which request/response shape a provider's API speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderApi {
    /// The `/chat/completions` and `/models` shape shared by OpenAI,
    /// OpenRouter, Hack Club AI, and Kimi (Moonshot).
    OpenAiCompatible,
    /// Anthropic's `/v1/messages` and `/v1/models` shape.
    Anthropic,
}

/// A provider pinel can send chat requests to.
pub struct ProviderInfo {
    /// Stable identifier used for credential storage - never shown in the
    /// UI and never changes, unlike `name`.
    pub id: &'static str,
    pub name: &'static str,
    /// Short label for the narrow provider column in the model picker,
    /// since there are no provider logo assets yet.
    pub badge: &'static str,
    /// Default base URL. Locally-hosted providers let the user override
    /// this (see `ProviderCredential::base_url`), since the host/port
    /// depends on how they've set up their local server.
    pub base_url: &'static str,
    pub api: ProviderApi,
    /// Whether this provider needs an API key to be usable. False for
    /// locally-hosted servers (Ollama, LM Studio, ...), which typically
    /// don't require auth.
    pub requires_api_key: bool,
}

pub const PROVIDERS: &[ProviderInfo] = &[
    ProviderInfo {
        id: "openai",
        name: "OpenAI",
        badge: "OA",
        base_url: "https://api.openai.com/v1",
        api: ProviderApi::OpenAiCompatible,
        requires_api_key: true,
    },
    ProviderInfo {
        id: "anthropic",
        name: "Anthropic",
        badge: "AN",
        base_url: "https://api.anthropic.com/v1",
        api: ProviderApi::Anthropic,
        requires_api_key: true,
    },
    ProviderInfo {
        id: "openrouter",
        name: "OpenRouter",
        badge: "OR",
        base_url: "https://openrouter.ai/api/v1",
        api: ProviderApi::OpenAiCompatible,
        requires_api_key: true,
    },
    ProviderInfo {
        id: "hackclub",
        name: "Hack Club AI",
        badge: "HC",
        base_url: "https://ai.hackclub.com/proxy/v1",
        api: ProviderApi::OpenAiCompatible,
        requires_api_key: true,
    },
    ProviderInfo {
        id: "kimi",
        name: "Kimi",
        badge: "KM",
        base_url: "https://api.moonshot.ai/v1",
        api: ProviderApi::OpenAiCompatible,
        requires_api_key: true,
    },
    ProviderInfo {
        id: "ollama",
        name: "Ollama",
        badge: "OL",
        base_url: "http://localhost:11434/v1",
        api: ProviderApi::OpenAiCompatible,
        requires_api_key: false,
    },
    ProviderInfo {
        id: "lmstudio",
        name: "LM Studio",
        badge: "LM",
        base_url: "http://localhost:1234/v1",
        api: ProviderApi::OpenAiCompatible,
        requires_api_key: false,
    },
    ProviderInfo {
        id: "custom",
        name: "Custom (OpenAI-Compatible)",
        badge: "CU",
        base_url: "http://localhost:8000/v1",
        api: ProviderApi::OpenAiCompatible,
        requires_api_key: false,
    },
];

/// Looks up a provider by its stable id (used for credential storage).
pub fn by_id(id: &str) -> Option<&'static ProviderInfo> {
    PROVIDERS.iter().find(|p| p.id == id)
}

/// Looks up a provider by its display name (used by chat sessions, which
/// store the provider name rather than its id).
pub fn by_name(name: &str) -> Option<&'static ProviderInfo> {
    PROVIDERS.iter().find(|p| p.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_provider_id_resolves() {
        assert_eq!(by_id("openai").map(|p| p.name), Some("OpenAI"));
    }

    #[test]
    fn unknown_provider_id_returns_none() {
        assert!(by_id("not-a-real-provider").is_none());
    }

    #[test]
    fn known_provider_name_resolves() {
        assert_eq!(by_name("Anthropic").map(|p| p.id), Some("anthropic"));
    }

    #[test]
    fn every_provider_id_is_unique() {
        let mut ids: Vec<&str> = PROVIDERS.iter().map(|p| p.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), PROVIDERS.len());
    }

    #[test]
    fn unknown_provider_name_returns_none() {
        assert!(by_name("Not A Real Provider").is_none());
    }

    #[test]
    fn every_remote_provider_base_url_uses_https() {
        // Local providers (Ollama, LM Studio, Custom) run on plain http on
        // the user's own machine - only hosted providers need https.
        for provider in PROVIDERS.iter().filter(|p| p.requires_api_key) {
            assert!(
                provider.base_url.starts_with("https://"),
                "{}'s base_url isn't https: {}",
                provider.name,
                provider.base_url
            );
        }
    }

    #[test]
    fn local_providers_do_not_require_an_api_key() {
        for id in ["ollama", "lmstudio", "custom"] {
            let provider = by_id(id).unwrap_or_else(|| panic!("{id} should be in PROVIDERS"));
            assert!(
                !provider.requires_api_key,
                "{} should not require a key",
                provider.name
            );
        }
    }

    #[test]
    fn hosted_providers_require_an_api_key() {
        for id in ["openai", "anthropic", "openrouter", "hackclub", "kimi"] {
            let provider = by_id(id).unwrap_or_else(|| panic!("{id} should be in PROVIDERS"));
            assert!(
                provider.requires_api_key,
                "{} should require a key",
                provider.name
            );
        }
    }

    #[test]
    fn every_provider_base_url_has_no_trailing_slash() {
        // ai_client builds request URLs as `{base_url}/models`, so a
        // trailing slash here would silently produce a double slash.
        for provider in PROVIDERS {
            assert!(
                !provider.base_url.ends_with('/'),
                "{}'s base_url ends with a slash: {}",
                provider.name,
                provider.base_url
            );
        }
    }
}
