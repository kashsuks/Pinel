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
    pub base_url: &'static str,
    pub api: ProviderApi,
}

pub const PROVIDERS: &[ProviderInfo] = &[
    ProviderInfo {
        id: "openai",
        name: "OpenAI",
        badge: "OA",
        base_url: "https://api.openai.com/v1",
        api: ProviderApi::OpenAiCompatible,
    },
    ProviderInfo {
        id: "anthropic",
        name: "Anthropic",
        badge: "AN",
        base_url: "https://api.anthropic.com/v1",
        api: ProviderApi::Anthropic,
    },
    ProviderInfo {
        id: "openrouter",
        name: "OpenRouter",
        badge: "OR",
        base_url: "https://openrouter.ai/api/v1",
        api: ProviderApi::OpenAiCompatible,
    },
    ProviderInfo {
        id: "hackclub",
        name: "Hack Club AI",
        badge: "HC",
        base_url: "https://ai.hackclub.com/proxy/v1",
        api: ProviderApi::OpenAiCompatible,
    },
    ProviderInfo {
        id: "kimi",
        name: "Kimi",
        badge: "KM",
        base_url: "https://api.moonshot.ai/v1",
        api: ProviderApi::OpenAiCompatible,
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
}
