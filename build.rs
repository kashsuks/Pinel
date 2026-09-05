//! Bakes `DISCORD_CLIENT_ID` into the binary at compile time (via
//! `option_env!("DISCORD_CLIENT_ID")` in `src/discord_rpc/client.rs`) so every
//! official build shares the same Discord application ID without the value
//! ever appearing in a committed source file.
//!
//! Resolution order:
//! 1. `DISCORD_CLIENT_ID` already set in the build environment (CI secret).
//! 2. A `.env` file at the repo root (gitignored; for local dev builds).
//!
//! If neither is present, the app falls back to a user-supplied ID from
//! `~/.config/pinel/discord.lua` at runtime (see `src/discord_rpc/config.rs`).

use std::{env, fs, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-env-changed=DISCORD_CLIENT_ID");

    if let Ok(id) = env::var("DISCORD_CLIENT_ID") {
        println!("cargo:rustc-env=DISCORD_CLIENT_ID={id}");
        return;
    }

    let dotenv_path = Path::new(".env");
    if let Ok(content) = fs::read_to_string(dotenv_path) {
        for line in content.lines() {
            let line = line.trim();
            if let Some(id) = line.strip_prefix("DISCORD_CLIENT_ID=") {
                println!("cargo:rustc-env=DISCORD_CLIENT_ID={}", id.trim());
                return;
            }
        }
    }
}
