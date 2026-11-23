//! Bakes `DISCORD_CLIENT_ID` into the binary at compile time (via
//! `option_env!("DISCORD_CLIENT_ID")` in `src/discord_rpc/client.rs`) so every
//! official build shares the same Discord application ID without the value
//! ever appearing in a committed source file.
//!
//! Resolution order:
//! 1. `DISCORD_CLIENT_ID` already set in the build environment (CI secret).
//! 2. A `.env` file at the repo root (gitignored; for local dev builds).
//!
//! An empty value counts as missing. If neither gives an ID, the build prints
//! a warning and the app falls back to a user-supplied ID from
//! `~/.config/pinel/discord.lua` at runtime (see `src/discord_rpc/config.rs`).
//! Without that file Rich Presence connects with no ID and Discord drops it.

use std::{env, fs, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-env-changed=DISCORD_CLIENT_ID");

    if let Some(id) = env::var("DISCORD_CLIENT_ID").ok().filter(|id| !id.trim().is_empty()) {
        println!("cargo:rustc-env=DISCORD_CLIENT_ID={}", id.trim());
        return;
    }

    // the reason goes in the warning, so a `.env` that exists but can't be
    // read (permissions, a sandbox) doesn't look the same as no `.env` at all
    let reason = match fs::read_to_string(Path::new(".env")) {
        Ok(content) => {
            let id = content
                .lines()
                .find_map(|line| line.trim().strip_prefix("DISCORD_CLIENT_ID="))
                .map(str::trim)
                .filter(|id| !id.is_empty());
            match id {
                Some(id) => {
                    println!("cargo:rustc-env=DISCORD_CLIENT_ID={id}");
                    return;
                },
                None => "`.env` has no non-empty DISCORD_CLIENT_ID line".to_string(),
            }
        },
        Err(err) => format!("could not read `.env`: {err}"),
    };

    println!(
        "cargo:warning=no Discord client ID baked in ({reason}). Rich Presence will \
         only work if ~/.config/pinel/discord.lua sets client_id"
    );
}
