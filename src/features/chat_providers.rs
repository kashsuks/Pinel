//! Static placeholder provider/model catalog for the chat tab's picker UI.
//!
//! This is intentionally not backed by any real API - there is no key
//! management or "attach a provider" flow yet (that is a separate,
//! follow-up feature). Real model lists fetched from each provider replace
//! this once a provider is actually wired up.

/// A provider and the models it offers, for display in the picker.
pub struct ProviderInfo {
    pub name: &'static str,
    pub models: &'static [&'static str],
}

pub const PROVIDERS: &[ProviderInfo] = &[
    ProviderInfo {
        name: "OpenAI",
        models: &["gpt-4o", "gpt-4o-mini", "o1-mini"],
    },
    ProviderInfo {
        name: "Anthropic",
        models: &["Claude Opus", "Claude Sonnet", "Claude Haiku"],
    },
    ProviderInfo {
        name: "Hack Club AI",
        models: &["Hack Club AI Default"],
    },
    ProviderInfo {
        name: "OpenRouter",
        models: &["OpenRouter Auto"],
    },
];

/// Looks up the placeholder model list for a provider name. Returns an
/// empty slice for an unknown (or not-yet-chosen) provider.
pub fn models_for(provider: &str) -> &'static [&'static str] {
    PROVIDERS.iter().find(|p| p.name == provider).map(|p| p.models).unwrap_or(&[])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_provider_returns_its_models() {
        assert_eq!(models_for("OpenAI"), &["gpt-4o", "gpt-4o-mini", "o1-mini"]);
    }

    #[test]
    fn unknown_provider_returns_empty() {
        assert!(models_for("Not A Real Provider").is_empty());
    }
}
