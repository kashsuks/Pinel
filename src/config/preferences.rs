use std::{fs, io::Write, path::PathBuf};

use super::theme_manager::{get_config_dir, load_theme, ThemeColors};

#[derive(Debug, Clone)]
pub struct EditorPreferences {
    pub first_launch: bool,
    pub tab_size: usize,
    pub use_spaces: bool,
    /// copy the leading whitespaces of the current line onto the new line on enter
    pub auto_indent_enabled: bool,
    pub vim_mode_enabled: bool,
    pub discord_rpc_enabled: bool,
    pub restore_session_enabled: bool,
    pub autosave_enabled: bool,
    pub autosave_interval_ms: u64,
    pub theme_name: String,
    pub window_width: f32,
    pub window_height: f32,
    /// Width of the line-number gutter in logical pixels (default 40).
    pub line_number_width: f32,
    pub tab_drag_floating: bool,
    /// Enable developer mode with debug logging
    #[cfg(feature = "unstable-comet")]
    pub developer_mode: bool,
}

impl Default for EditorPreferences {
    fn default() -> Self {
        Self {
            first_launch: true,
            tab_size: 4,
            use_spaces: true,
            auto_indent_enabled: true,
            vim_mode_enabled: false,
            discord_rpc_enabled: false,
            restore_session_enabled: true,
            autosave_enabled: true,
            autosave_interval_ms: 300,
            theme_name: "Pinel Blueberry Dark".to_string(),
            window_width: 1200.0,
            window_height: 800.0,
            line_number_width: 40.0,
            tab_drag_floating: true,
            #[cfg(feature = "unstable-comet")]
            developer_mode: false,
        }
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn get_themes_dir() -> PathBuf {
    get_config_dir().join("themes")
}

pub fn load_preferences() -> EditorPreferences {
    let primary = get_preferences_path();
    let legacy = legacy_preferences_path();

    let primary_prefs = read_preferences_from(&primary);
    let legacy_prefs = legacy.as_ref().and_then(read_preferences_from);

    match (primary_prefs, legacy_prefs) {
        (Some(prefs), None) => prefs,
        (None, Some(prefs)) => {
            let _ = save_preferences_to_path(&prefs, &primary);
            prefs
        },
        (Some(primary_prefs), Some(legacy_prefs)) => {
            if legacy_is_newer_than_primary(legacy.as_ref(), &primary) {
                let _ = save_preferences_to_path(&legacy_prefs, &primary);
                legacy_prefs
            } else {
                primary_prefs
            }
        },
        (None, None) => {
            let prefs = EditorPreferences::default();
            let _ = save_preferences_to_path(&prefs, &primary);
            prefs
        },
    }
}

fn legacy_preferences_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".config").join("pinel").join("preferences.lua"))
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
            let value = value.trim().trim_end_matches(',').trim_matches('"').trim_matches('\'');
            match key {
                "tab_size" => {
                    if let Ok(size) = value.parse::<usize>() {
                        prefs.tab_size = size.clamp(1, 16);
                    }
                },
                "use_spaces" => {
                    prefs.use_spaces = value == "true";
                },
                "auto_indent_enabled" => {
                    prefs.auto_indent_enabled = value == "true";
                },
                "vim_mode_enabled" => {
                    prefs.vim_mode_enabled = value == "true";
                },
                "discord_rpc_enabled" => {
                    prefs.discord_rpc_enabled = value == "true";
                },
                "restore_session_enabled" => {
                    prefs.restore_session_enabled = value == "true";
                },
                "autosave_enabled" => {
                    prefs.autosave_enabled = value == "true";
                },
                "autosave_interval_ms" => {
                    if let Ok(interval) = value.parse::<u64>() {
                        prefs.autosave_interval_ms = interval.clamp(30, 1000);
                    }
                },
                "theme_name" => {
                    prefs.theme_name = value.to_string();
                },
                "window_width" => {
                    if let Ok(width) = value.parse::<f32>() {
                        prefs.window_width = width.clamp(640.0, 10000.0);
                    }
                },
                "window_height" => {
                    if let Ok(height) = value.parse::<f32>() {
                        prefs.window_height = height.clamp(480.0, 10000.0);
                    }
                },
                "line_number_width" => {
                    if let Ok(w) = value.parse::<f32>() {
                        prefs.line_number_width = w.clamp(20.0, 120.0);
                    }
                },
                "first_launch" => {
                    prefs.first_launch = value == "true";
                },
                "tab_drag_floating" => {
                    prefs.tab_drag_floating = value == "true";
                },
                #[cfg(feature = "unstable-comet")]
                "developer_mode" => {
                    prefs.developer_mode = value == "true";
                },
                _ => {},
            }
        }
    }
    prefs
}

pub fn save_preferences(prefs: &EditorPreferences) -> Result<(), std::io::Error> {
    let path = get_preferences_path();
    save_preferences_to_path(prefs, &path)
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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
    first_launch = {},
    tab_size = {},
    use_spaces = {},
    auto_indent_enabled = {},
    vim_mode_enabled = {},
    discord_rpc_enabled = {},
    restore_session_enabled = {},
    autosave_enabled = {},
    -- Autosave interval in milliseconds (30–1000)
    autosave_interval_ms = {},
    theme_name = "{}",
    window_width = {},
    window_height = {},
    -- Width of the line-number gutter in logical pixels (20–120)
    line_number_width = {},
    -- Tab drag style: true = floating ghost (default), false = static shift
    tab_drag_floating = {},
{}}}
"#,
        prefs.first_launch,
        prefs.tab_size,
        prefs.use_spaces,
        prefs.auto_indent_enabled,
        prefs.vim_mode_enabled,
        prefs.discord_rpc_enabled,
        prefs.restore_session_enabled,
        prefs.autosave_enabled,
        prefs.autosave_interval_ms,
        prefs.theme_name,
        prefs.window_width,
        prefs.window_height,
        prefs.line_number_width,
        prefs.tab_drag_floating,
        developer_mode_config,
    );
    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    #[test]
    fn parse_preferences_empty_returns_defaults() {
        let prefs = parse_preferences("");
        let defaults = EditorPreferences::default();
        assert_eq!(prefs.tab_size, defaults.tab_size);
        assert_eq!(prefs.use_spaces, defaults.use_spaces);
        assert_eq!(prefs.theme_name, defaults.theme_name);
    }

    #[test]
    fn parse_preferences_reads_known_keys() {
        let content = r#"
            return {
                tab_size = 2,
                use_spaces = false,
                auto_indent_enabled = false,
                vim_mode_enabled = true,
                theme_name = "Solarized",
                window_width = 1000.0,
                window_height = 700.0,
            }
        "#;
        let prefs = parse_preferences(content);
        assert_eq!(prefs.tab_size, 2);
        assert!(!prefs.use_spaces);
        assert!(!prefs.auto_indent_enabled);
        assert!(prefs.vim_mode_enabled);
        assert_eq!(prefs.theme_name, "Solarized");
        assert_eq!(prefs.window_width, 1000.0);
        assert_eq!(prefs.window_height, 700.0);
    }

    #[test]
    fn parse_preferences_clamps_out_of_range_values() {
        let content = r#"
            tab_size = 99,
            autosave_interval_ms = 5,
            window_width = 1.0,
            window_height = 1.0,
            line_number_width = 500.0,
        "#;
        let prefs = parse_preferences(content);
        assert_eq!(prefs.tab_size, 16); // clamped to max
        assert_eq!(prefs.autosave_interval_ms, 30); // clamped to min
        assert_eq!(prefs.window_width, 640.0); // clamped to min
        assert_eq!(prefs.window_height, 480.0); // clamped to min
        assert_eq!(prefs.line_number_width, 120.0); // clamped to max
    }

    #[test]
    fn parse_preferences_ignores_comments_and_unknown_keys() {
        let content = r#"
            -- this is a comment
            return {
            unknown_key = "whatever",
            tab_size = 8,
            }
        "#;
        let prefs = parse_preferences(content);
        assert_eq!(prefs.tab_size, 8);
    }

    #[test]
    fn indent_unit_returns_spaces_when_uses_spaces_true() {
        let mut prefs = EditorPreferences::default();
        prefs.use_spaces = true;
        prefs.tab_size = 3;
        assert_eq!(prefs.indent_unit(), "   ");
    }

    #[test]
    fn indent_unit_returns_tab_when_use_spaces_false() {
        let mut prefs = EditorPreferences::default();
        prefs.use_spaces = false;
        assert_eq!(prefs.indent_unit(), "\t");
    }

    #[test]
    fn legacy_is_newer_than_primary_true_when_legacy_has_later_mtime() {
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("legacy.lua");
        let primary = dir.path().join("primary.lua");
        fs::write(&primary, "").unwrap();
        fs::write(&legacy, "").unwrap();

        // force legacys mtime to be strictly after primarys
        let future = SystemTime::now() + Duration::from_secs(10);
        let file = fs::OpenOptions::new().write(true).open(&legacy).unwrap();
        file.set_modified(future).unwrap();

        assert!(legacy_is_newer_than_primary(Some(&legacy), &primary));
    }

    #[test]
    fn legacy_is_newer_than_primary_false_when_legacy_missing() {
        let dir = tempfile::tempdir().unwrap();
        let primary = dir.path().join("primary.lua");
        fs::write(&primary, "").unwrap();
        assert!(!legacy_is_newer_than_primary(None, &primary));
    }

    #[test]
    fn legacy_is_newer_than_primary_true_when_primary_missing() {
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join("legacy.lua");
        let primary = dir.path().join("does_not_exist.lua");
        fs::write(&legacy, "").unwrap();
        assert!(legacy_is_newer_than_primary(Some(&legacy), &primary));
    }

    #[test]
    fn save_then_read_preferences_roundtrips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("preferences.lua");

        let mut prefs = EditorPreferences::default();
        prefs.tab_size = 2;
        prefs.use_spaces = false;
        prefs.theme_name = "Custom Theme".to_string();

        save_preferences_to_path(&prefs, &path).unwrap();
        let loaded = read_preferences_from(&path).expect("file should parse");

        assert_eq!(loaded.tab_size, prefs.tab_size);
        assert_eq!(loaded.use_spaces, prefs.use_spaces);
        assert_eq!(loaded.theme_name, prefs.theme_name);
    }

    #[test]
    fn read_preferences_from_missing_file_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nope.lua");
        assert!(read_preferences_from(&path).is_none());
    }
}
