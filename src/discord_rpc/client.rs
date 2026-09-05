//! A thin wrapper around the `discord-rich-presence` crate's IPC client.
//!
//! This module only knows how to talk to Discord over its local IPC socket;
//! it has no knowledge of Pinel's editor state, tabs, or the active file
//! tracker (see [`crate::features::activity_state`] for that).

use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

use super::config;

pub struct DiscordRpcClient {
    inner: DiscordIpcClient,
    connected: bool,
}

/// The large image slot on the Rich Presence card, and the tooltip shown
/// when hovering over it. `key` must match an asset key already uploaded
/// under this application's Rich Presence -> Art Assets in the Discord
/// Developer Portal - unrecognized keys just render as a blank square.
pub struct LargeImage<'a> {
    pub key: &'a str,
    pub text: &'a str,
}

impl DiscordRpcClient {
    /// Constructs the new client without connecting to anything yet.
    ///
    /// The application client ID is baked in at compile time from the
    /// `DISCORD_CLIENT_ID` build environment variable (see `build.rs`), so
    /// every official build shares the same ID without it ever appearing in
    /// a committed source file. Builds without that variable set (e.g.
    /// building from source without the CI secret) fall back to a
    /// user-supplied ID in `~/.config/pinel/discord.lua` (see [`config`]).
    pub fn new() -> Self {
        let client_id = option_env!("DISCORD_CLIENT_ID")
            .filter(|id| !id.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| config::load().client_id);
        Self {
            inner: DiscordIpcClient::new(&client_id),
            connected: false,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Attempts to connect to the local Discord IPC socket. Returns `false`
    /// (not an error) if Discord isn't running or isn't reachable.
    pub fn connect(&mut self) -> bool {
        self.connected = self.inner.connect().is_ok();
        self.connected
    }

    /// Sets the current Rich Presence activity. No-ops (returns `false`)
    /// if not currently connected - callers should check [`Self::connect`]
    ///
    /// `state` is omitted from the payload entirely when `None` - Discord's
    /// Rich Presence protocol treats an empty string as an invalid field
    /// rather than "no state", so passing `Some("")` would silently drop
    /// the whole activity update.
    pub fn set_presence(
        &mut self,
        details: &str,
        state: Option<&str>,
        large_image: Option<LargeImage>,
        started_at_unix_ms: i64,
    ) -> bool {
        if !self.connected {
            return false;
        }

        let mut activity = activity::Activity::new()
            .details(details)
            .timestamps(activity::Timestamps::new().start(started_at_unix_ms));
        if let Some(state) = state {
            activity = activity.state(state);
        }
        if let Some(LargeImage { key, text }) = large_image {
            activity = activity.assets(
                activity::Assets::new()
                    .large_image(key)
                    .large_text(text),
            );
        }

        if self.inner.set_activity(activity).is_err() {
            // The connection may have dropped (e.g. discord closed);
            self.connected = false;
            return false;
        }

        true
    }

    pub fn clear_presence(&mut self) {
        if self.connected {
            let _ = self.inner.clear_activity();
        }
    }
}

impl Default for DiscordRpcClient {
    fn default() -> Self {
        Self::new()
    }
}
