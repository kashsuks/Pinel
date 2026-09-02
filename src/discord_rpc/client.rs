//! A thin wrapper around the `discord-rich-presence` crate's IPC client.
//!
//! This module only knows how to talk to Discord over its local IPC socket
//! it has no knowledge of Pinel's editor state, tabs, or the active0file
//! tracker (see [`crate::features::activity_state`] for that).

use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

const DISCORD_CLIENT_ID: &str = "dummy_value_set_later";

pub struct DiscordRpcClient {
    inner: DiscordIpcClient,
    connected: bool,
}

impl DiscordRpcClient {
    /// Constructs the new client without connecting to anything yet.
    pub fn new() -> Self {
        Self {
            inner: DiscordIpcClient::new(DISCORD_CLIENT_ID),
            connected: false,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Attempts to connect to the local Discord ICP socket. Returns `false`
    /// (not an error) if Discord isn't runing or isn't reachable.
    pub fn connect(&mut self) -> bool {
        self.connected = self.inner.connect().is_ok();
        self.connected
    }

    /// Sets the current Rich Presence activity. No-ops (returns `false`)
    /// if not currently connected - callers should check [`Self::connect`]
    pub fn set_presence(&mut self, details: &str, state: &str, started_at_unix_ms: i64) -> bool {
        if !self.connected {
            return false;
        }

        let activity = activity::Activity::new()
            .details(details)
            .state(state)
            .timestamps(activity::Timestamps::new().start(started_at_unix_ms));

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
