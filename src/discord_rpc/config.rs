//! Loads the Discord application (client) ID from the user's local config
//! directory rather than compiling it into the source tree, mirroring how
//! `wakatime::config` keeps the WakaTime API key out of git.

use std::{fs, io::Write, path::PathBuf};

use crate::config::theme_manager::get_config_dir;

#[derive(Debug, Clone, Default)]
pub struct DiscordConfig {
    pub client_id: String,
}

fn get_discord_path() -> PathBuf {
    get_config_dir().join("discord.lua")
}

pub fn load() -> DiscordConfig {
    let path = get_discord_path();
    if let Ok(content) = fs::read_to_string(&path) {
        from_lua(&content)
    } else {
        DiscordConfig::default()
    }
}

#[allow(dead_code)]
pub fn save(cfg: &DiscordConfig) -> Result<(), std::io::Error> {
    let path = get_discord_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::File::create(path)?;
    file.write_all(to_lua(cfg).as_bytes())?;
    Ok(())
}

fn to_lua(cfg: &DiscordConfig) -> String {
    format!("return {{\n client_id = \"{}\",\n}}\n", cfg.client_id)
}

fn from_lua(content: &str) -> DiscordConfig {
    let mut cfg = DiscordConfig::default();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("--") || line.starts_with("return") || line == "}" {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value
                .trim()
                .trim_end_matches(',')
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();

            if key == "client_id" {
                cfg.client_id = value;
            }
        }
    }

    cfg
}
