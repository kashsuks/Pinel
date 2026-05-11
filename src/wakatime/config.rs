/// This file is responsible for defining methods and processes
/// For the configuration files of `Pinel`
///
/// This includes (but is not restricted to):
/// - Wakatime/Hackatime CLI Config
/// - Config file (.config/pinel) location
/// - Parsing and/or writing to and from lua to theme code

use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WakaTimeConfig {
    pub api_key: String,
    pub api_url: String,
}

impl Default for WakaTimeConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_url: "https://api.wakatime.com/api/v1".to_string(), // change to hackatime/v1 for
                                                                    // hackatime support
        }
    }
}

fn get_config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("pinel")
}

fn get_wakatime_path() -> PathBuf {
    get_config_dir().join("wakatime.lua") // as mentioned before the path is changed here for
                                          // hackatime/v1
}

pub fn load() -> WakaTimeConfig {
    let path = get_wakatime_path();
    if let Ok(content) = fs::read_to_string(&path) {
        from_lua(&content).unwrap_or_default()
    } else {
        WakaTimeConfig::default()
    }
}

pub fn save(cfg: &WakaTimeConfig) -> Result<(), std::io::Error> {
    let path = get_wakatime_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::File::create(path)?;
    file.write_all(to_lua(cfg).as_bytes())?;
    Ok(())
}

fn to_lua(cfg: &WakaTimeConfig) -> String {
    format!(
        "return {{\n api_key = \"{}\",\n api_url = \"{}\",\n}}\n",
        cfg.api_key, cfg.api_url
    )
}

/// Responsible for parsing lua theme code to wakatime config
///
/// Arguments:
/// - content(str): The contents of the lua file that will be parsed into wakatime config
///
/// Returns:
/// - Result<WakatimeConfig, String>: The final wakatime config as a string with the
///                                   wakatime url and extra info
fn from_lua(content: &str) -> Result<WakaTimeConfig, String> {
    let mut cfg = WakaTimeConfig::default();

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

            match key {
                "api_key" => cfg.api_key = value, // secret, keep private
                "api_url" => cfg.api_url = value,
                _ => {}
            }
        }
    }

    Ok(cfg)
}
