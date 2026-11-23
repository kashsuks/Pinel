//! Release discovery and persistent notification choices.

use anyhow::{Context, Result};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Version embedded in this executable, including its release channel.
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const RELEASES_URL: &str = "https://api.github.com/repos/kashsuks/pinel/releases";
const DAY: u64 = 24 * 60 * 60;

/// A published release newer than the installed executable.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateInfo {
    /// Semantic version without the tag's leading `v`.
    pub version: String,
    /// Browser URL for notes and installation instructions.
    pub url: String,
}

/// App-managed update bookkeeping, independent of the workspace session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdateState {
    /// Whether startup may contact GitHub for updates.
    pub disabled: bool,
    /// Earliest next automatic network check, as Unix seconds.
    pub next_check: u64,
    /// Most recently discovered update, retained between checks.
    pub available: Option<UpdateInfo>,
    /// Release explicitly skipped by the user.
    pub skipped_version: Option<String>,
    /// Release whose notification was postponed.
    pub reminded_version: Option<String>,
    /// End of the reminder delay, as Unix seconds.
    pub remind_until: u64,
}

impl UpdateState {
    /// Loads saved choices; missing or corrupt state falls back to defaults.
    pub fn load() -> Self {
        std::fs::read(crate::config::theme_manager::get_config_dir().join("updates.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    /// Saves atomically so interrupted writes do not erase user choices.
    pub fn save(&self) -> Result<()> {
        let path = crate::config::theme_manager::get_config_dir().join("updates.json");
        std::fs::create_dir_all(path.parent().context("update state has no parent")?)?;
        crate::fs_util::write_atomic(&path, serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }

    /// Whether a startup check is due. Manual checks bypass this policy.
    pub fn check_due(&self, now: u64) -> bool {
        !self.disabled && now >= self.next_check
    }

    /// Returns an eligible cached notification without making a request.
    pub fn notification(&self, current: &str, now: u64) -> Option<UpdateInfo> {
        let info = self.available.as_ref()?;
        if self.disabled
            || !is_newer(&info.version, current)
            || !channel_allows(&info.version, current)
            || self.skipped_version.as_deref() == Some(&info.version)
            || (self.reminded_version.as_deref() == Some(&info.version) && now < self.remind_until)
        {
            return None;
        }
        Some(info.clone())
    }

    /// Records a successful check, or backs off briefly after network failure.
    pub fn record_check(&mut self, result: &Result<Option<UpdateInfo>, String>, now: u64) {
        self.next_check = now.saturating_add(if result.is_ok() {
            DAY
        } else {
            15 * 60
        });
        if let Ok(info) = result {
            self.available = info.clone();
        }
    }

    /// Postpones only this release for one day.
    pub fn remind_tomorrow(&mut self, version: String, now: u64) {
        self.reminded_version = Some(version);
        self.remind_until = now.saturating_add(DAY);
    }
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
}

/// Current wall-clock time for persisted cooldowns.
pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(format!("pinel/{CURRENT_VERSION}"))
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .context("could not create release client")
}

/// Finds a newer published release in the installed build's channel.
/// Errors remain distinct from an up-to-date result for manual checks.
pub async fn check_for_update() -> Result<Option<UpdateInfo>> {
    let client = client()?;
    let response = client.get(format!("{RELEASES_URL}/latest")).send().await?;
    let mut releases = Vec::new();
    if response.status() != reqwest::StatusCode::NOT_FOUND {
        releases.push(response.error_for_status()?.json::<GithubRelease>().await?);
    }
    let current = Version::parse(CURRENT_VERSION)?;
    if !current.pre.is_empty() {
        releases.extend(
            client
                .get(format!("{RELEASES_URL}?per_page=100"))
                .send()
                .await?
                .error_for_status()?
                .json::<Vec<GithubRelease>>()
                .await?,
        );
    }
    Ok(select_release(releases, CURRENT_VERSION))
}

fn select_release(releases: Vec<GithubRelease>, current: &str) -> Option<UpdateInfo> {
    releases
        .into_iter()
        .filter(|r| !r.draft)
        .filter(|r| !r.prerelease || Version::parse(current).is_ok_and(|v| !v.pre.is_empty()))
        .filter_map(|r| {
            let version = r.tag_name.strip_prefix('v').unwrap_or(&r.tag_name);
            let parsed = Version::parse(version).ok()?;
            (is_newer(version, current) && channel_allows(version, current)).then_some((
                parsed,
                UpdateInfo {
                    version: version.to_owned(),
                    url: r.html_url,
                },
            ))
        })
        .max_by(|(a, _), (b, _)| a.cmp_precedence(b))
        .map(|(_, info)| info)
}

fn channel_allows(candidate: &str, current: &str) -> bool {
    match (Version::parse(candidate), Version::parse(current)) {
        (Ok(candidate), Ok(current)) => candidate.pre.is_empty() || !current.pre.is_empty(),
        _ => false,
    }
}

fn is_newer(candidate: &str, current: &str) -> bool {
    match (Version::parse(candidate), Version::parse(current)) {
        (Ok(candidate), Ok(current)) => candidate.cmp_precedence(&current).is_gt(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_semantic_versions() {
        for (candidate, current, expected) in [
            ("0.10.0", "0.9.0", true),
            ("1.0.0-dev.10", "1.0.0-dev.9", true),
            ("1.0.0", "1.0.0-dev.9", true),
            ("1.0.0-dev.9", "1.0.0", false),
            ("1.0.0+new", "1.0.0+old", false),
            ("invalid", "0.9.0", false),
            ("0.8.0", "0.9.0", false),
        ] {
            assert_eq!(is_newer(candidate, current), expected);
        }
    }

    #[test]
    fn test_select_release_channel_and_drafts() -> Result<()> {
        let releases = || {
            serde_json::from_str::<Vec<GithubRelease>>(
                r#"[
            {"tag_name":"v1.1.0-dev.10","html_url":"dev","prerelease":true},
            {"tag_name":"v2.0.0","html_url":"draft","draft":true},
            {"tag_name":"v1.0.0","html_url":"stable"},
            {"tag_name":"broken","html_url":"invalid"}
        ]"#,
            )
        };
        assert_eq!(
            select_release(releases()?, "0.9.0").context("stable")?.version,
            "1.0.0"
        );
        assert_eq!(
            select_release(releases()?, "1.1.0-dev.9").context("dev")?.version,
            "1.1.0-dev.10"
        );
        assert!(select_release(releases()?, "1.1.0").is_none());
        Ok(())
    }

    #[test]
    fn test_update_state_reminders_skip_and_cooldowns() -> Result<()> {
        let mut state = UpdateState::default();
        let info = UpdateInfo {
            version: "1.0.0".into(),
            url: "release".into(),
        };
        state.record_check(&Ok(Some(info.clone())), 100);
        assert!(!state.check_due(101));
        assert!(state.check_due(100 + DAY));
        state.remind_tomorrow(info.version.clone(), 100);
        assert!(state.notification("0.9.0", 101).is_none());
        assert!(state.notification("0.9.0", 100 + DAY).is_some());
        state.skipped_version = Some(info.version);
        assert!(state.notification("0.9.0", 100 + DAY).is_none());
        state.available.as_mut().context("cached update")?.version = "1.1.0".into();
        assert!(state.notification("0.9.0", 101).is_some());
        assert!(state.notification("1.1.0", 101).is_none());
        let restored: UpdateState = serde_json::from_str(&serde_json::to_string(&state)?)?;
        assert_eq!(restored.skipped_version, state.skipped_version);
        state.record_check(&Err("offline".into()), 200);
        assert_eq!(state.next_check, 1100);
        assert!(state.available.is_some());
        state.disabled = true;
        assert!(!state.check_due(u64::MAX));
        assert!(state.notification("0.9.0", u64::MAX).is_none());
        Ok(())
    }
}
