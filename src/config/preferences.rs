use super::theme_manager::{get_config_dir, load_theme, ThemeColors};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct EditorPreferences {
    pub tab_size: usize,
    pub use_spaces: bool,
    pub autosave_enabled: bool,
    pub autosave_interval_ms: u64,
    pub theme_name: String,
    pub window_width: f32,
    pub window_height: f32,
    /// Width of the line-number gutter in logical pixels (default 40).
    pub line_number_width: f32,
    /// Enable developer mode with debug logging
    #[cfg(feature = "unstable-comet")]
    pub developer_mode: bool,
}

impl Default for EditorPreferences {
    fn default() -> Self {
        Self {
            tab_size: 4,
            use_spaces: true,
            autosave_enabled: true,
            autosave_interval_ms: 300,
            theme_name: "Pinel Blueberry Dark".to_string(),
            window_width: 1200.0,
            window_height: 800.0,
            line_number_width: 40.0,
            #[cfg(feature = "unstable-comet")]
            developer_mode: false,
        }
    }
}

impl EditorPreferences {
    pub fn indent_unit(&self) -> String {
        if self.use_spaces {
            " ".repeat(self.tab_size)
        } else {
            "\t".to_string()
        }
    }
}

pub fn get_preferences_path() -> PathBuf {
    get_config_dir().join("preferences.lua")
}

pub fn get_themes_dir() -> PathBuf {
    get_config_dir().join("themes")
}

pub fn load_preferences() -> EditorPreferences {
    let primary = get_preferences_path();
    let legacy = legacy_preferences_path();

    let primary_prefs = read_preferences_from(&primary);
    let legacy_prefs = legacy.as_ref().and_then(|path| read_preferences_from(path));

    match (primary_prefs, legacy_prefs) {
        (Some(prefs), None) => prefs,
        (None, Some(prefs)) => {
            let _ = save_preferences_to_path(&prefs, &primary);
            prefs
        }
        (Some(primary_prefs), Some(legacy_prefs)) => {
            if legacy_is_newer_than_primary(legacy.as_ref(), &primary) {
                let _ = save_preferences_to_path(&legacy_prefs, &primary);
                legacy_prefs
            } else {
                primary_prefs
            }
        }
        (None, None) => {
            let prefs = EditorPreferences::default();
            let _ = save_preferences_to_path(&prefs, &primary);
            prefs
        }
    }
}

fn legacy_preferences_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(
        PathBuf::from(home)
            .join(".config")
            .join("pinel")
            .join("preferences.lua"),
    )
}

fn parse_preferences(content: &str) -> EditorPreferences {
    let mut prefs = EditorPreferences::default();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("--") || line == "return {" || line == "}" {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value
                .trim()
                .trim_end_matches(',')
                .trim_matches('"')
                .trim_matches('\'');
            match key {
                "tab_size" => {
                    if let Ok(size) = value.parse::<usize>() {
                        prefs.tab_size = size.max(1).min(16);
                    }
                }
                "use_spaces" => {
                    prefs.use_spaces = value == "true";
                }
                "autosave_enabled" => {
                    prefs.autosave_enabled = value == "true";
                }
                "autosave_interval_ms" => {
                    if let Ok(interval) = value.parse::<u64>() {
                        prefs.autosave_interval_ms = interval.clamp(30, 1000);
                    }
                }
                "theme_name" => {
                    prefs.theme_name = value.to_string();
                }
                "window_width" => {
                    if let Ok(width) = value.parse::<f32>() {
                        prefs.window_width = width.max(640.0).min(10000.0);
                    }
                }
                "window_height" => {
                    if let Ok(height) = value.parse::<f32>() {
                        prefs.window_height = height.max(480.0).min(10000.0);
                    }
                }
                "line_number_width" => {
                    if let Ok(w) = value.parse::<f32>() {
                        prefs.line_number_width = w.max(20.0).min(120.0);
                    }
                }
                #[cfg(feature = "unstable-comet")]
                "developer_mode" => {
                    prefs.developer_mode = value == "true";
                }
                _ => {}
            }
        }
    }
    prefs
}

pub fn save_preferences(prefs: &EditorPreferences) -> Result<(), std::io::Error> {
    let path = get_preferences_path();
    save_preferences_to_path(prefs, &path)
}

pub fn list_available_themes() -> Vec<String> {
    let mut themes = vec!["default".to_string()];
    let themes_dir = get_themes_dir();
    if let Ok(entries) = fs::read_dir(&themes_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".lua") {
                    themes.push(name.trim_end_matches(".lua").to_string());
                }
            }
        }
    }
    themes
}

pub fn load_theme_by_name(name: &str) -> ThemeColors {
    if name == "default" {
        return ThemeColors::default();
    }

    let theme_path = get_themes_dir().join(format!("{}.lua", name));
    if let Ok(content) = fs::read_to_string(&theme_path) {
        if let Ok(theme) = ThemeColors::from_lua(&content) {
            return theme;
        }
    }

    load_theme()
}

fn read_preferences_from(path: &PathBuf) -> Option<EditorPreferences> {
    let content = fs::read_to_string(path).ok()?;
    Some(parse_preferences(&content))
}

fn legacy_is_newer_than_primary(legacy: Option<&PathBuf>, primary: &PathBuf) -> bool {
    let Some(legacy) = legacy else {
        return false;
    };
    let legacy_meta = fs::metadata(legacy).ok();
    let primary_meta = fs::metadata(primary).ok();
    let legacy_mtime = legacy_meta.and_then(|m| m.modified().ok());
    let primary_mtime = primary_meta.and_then(|m| m.modified().ok());
    match (legacy_mtime, primary_mtime) {
        (Some(legacy_mtime), Some(primary_mtime)) => legacy_mtime > primary_mtime,
        (Some(_), None) => true,
        _ => false,
    }
}

fn save_preferences_to_path(
    prefs: &EditorPreferences,
    path: &PathBuf,
) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    #[cfg(feature = "unstable-comet")]
    let developer_mode_config = format!(
        "    -- Enable developer mode with debug logging (WARNING: Logs may contain sensitive data)\n    developer_mode = {},\n",
        prefs.developer_mode
    );
    #[cfg(not(feature = "unstable-comet"))]
    let developer_mode_config = String::new();

    let content = format!(
        r#"-- Pinel Editor Preferences
-- Edit these values to customize your editor

return {{
    tab_size = {},
    use_spaces = {},
    autosave_enabled = {},
    -- Autosave interval in milliseconds (30–1000)
    autosave_interval_ms = {},
    theme_name = "{}",
    window_width = {},
    window_height = {},
    -- Width of the line-number gutter in logical pixels (20–120)
    line_number_width = {},
{}}}
"#,
        prefs.tab_size,
        prefs.use_spaces,
        prefs.autosave_enabled,
        prefs.autosave_interval_ms,
        prefs.theme_name,
        prefs.window_width,
        prefs.window_height,
        prefs.line_number_width,
        developer_mode_config,
    );
    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
