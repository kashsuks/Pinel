//! Launch-at-login registration for the running Pinel binary.
//!
//! Each platform uses its native per-user mechanism, so no extra crates or
//! admin rights are needed:
//! - macOS: a LaunchAgent plist in `~/Library/LaunchAgents`
//! - Linux and other freedesktop systems: a `.desktop` entry in `~/.config/autostart`
//! - Windows: a value under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`

use std::io;
use std::path::Path;

/// LaunchAgent label, matching `CFBundleIdentifier` in `src/assets/macos/Info.plist`.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
const MACOS_LABEL: &str = "com.kashsuks.pinel";

/// Registers (`enabled = true`) or removes (`enabled = false`) the entry that
/// starts the current executable at login. Removing an entry that does not
/// exist is not an error.
pub fn set_enabled(enabled: bool) -> io::Result<()> {
    let exe = std::env::current_exe()?;
    set_enabled_for(&exe, enabled)
}

#[cfg(target_os = "macos")]
fn set_enabled_for(exe: &Path, enabled: bool) -> io::Result<()> {
    let path = home_dir()?.join("Library/LaunchAgents").join(format!("{MACOS_LABEL}.plist"));
    if enabled {
        write_file(&path, &macos_plist(exe))
    } else {
        remove_file(&path)
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn set_enabled_for(exe: &Path, enabled: bool) -> io::Result<()> {
    let dir = dirs::config_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no config directory"))?;
    let path = dir.join("autostart").join("pinel.desktop");
    if enabled {
        write_file(&path, &desktop_entry(exe))
    } else {
        remove_file(&path)
    }
}

#[cfg(windows)]
fn set_enabled_for(exe: &Path, enabled: bool) -> io::Result<()> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    // Keep reg.exe from flashing a console window.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";

    let mut cmd = Command::new("reg");
    if enabled {
        let value = format!("\"{}\"", exe.display());
        cmd.args([
            "add", RUN_KEY, "/v", "Pinel", "/t", "REG_SZ", "/d", &value, "/f",
        ]);
    } else {
        cmd.args(["delete", RUN_KEY, "/v", "Pinel", "/f"]);
    }
    let status = cmd.creation_flags(CREATE_NO_WINDOW).status()?;
    // `reg delete` fails when the value is already absent, which is the goal.
    if status.success() || !enabled {
        Ok(())
    } else {
        Err(io::Error::other(format!("reg add exited with {status}")))
    }
}

#[cfg(not(any(unix, windows)))]
fn set_enabled_for(_exe: &Path, _enabled: bool) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "launch at login is not supported on this platform",
    ))
}

#[cfg(target_os = "macos")]
fn home_dir() -> io::Result<std::path::PathBuf> {
    dirs::home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no home directory"))
}

#[cfg_attr(not(unix), allow(dead_code))]
fn write_file(path: &Path, contents: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    crate::fs_util::write_atomic(path, contents)
}

#[cfg_attr(not(unix), allow(dead_code))]
fn remove_file(path: &Path) -> io::Result<()> {
    match std::fs::remove_file(path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

/// LaunchAgent that runs `exe` once at login.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn macos_plist(exe: &Path) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{MACOS_LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
"#,
        xml_escape(&exe.to_string_lossy())
    )
}

/// freedesktop autostart entry that runs `exe` at login.
#[cfg_attr(any(not(unix), target_os = "macos"), allow(dead_code))]
fn desktop_entry(exe: &Path) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName=Pinel\nExec={}\nX-GNOME-Autostart-enabled=true\n",
        desktop_exec_quote(&exe.to_string_lossy())
    )
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Quotes a path for a desktop entry `Exec=` key, per the Desktop Entry spec:
/// `"`, `` ` ``, `$` and `\` are backslash-escaped inside double quotes, and a
/// literal `%` is written as `%%` so it is not read as a field code.
#[cfg_attr(any(not(unix), target_os = "macos"), allow(dead_code))]
fn desktop_exec_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' | '`' | '$' | '\\' => {
                out.push('\\');
                out.push(c);
            },
            '%' => out.push_str("%%"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_plist_escapes_path() {
        let plist = macos_plist(Path::new("/Apps/A & B/<pinel>"));
        assert!(plist.contains("<string>/Apps/A &amp; B/&lt;pinel&gt;</string>"));
        assert!(plist.contains("<string>com.kashsuks.pinel</string>"));
        assert!(plist.contains("<key>RunAtLoad</key>\n    <true/>"));
    }

    #[test]
    fn test_desktop_exec_quote_special_characters() {
        assert_eq!(
            desktop_exec_quote("/opt/my app/pinel"),
            "\"/opt/my app/pinel\""
        );
        assert_eq!(desktop_exec_quote(r#"/a"$`\%b"#), r#""/a\"\$\`\\%%b""#);
    }

    #[test]
    fn test_desktop_entry_has_exec_line() {
        let entry = desktop_entry(Path::new("/usr/bin/pinel"));
        assert!(entry.starts_with("[Desktop Entry]\n"));
        assert!(entry.contains("\nExec=\"/usr/bin/pinel\"\n"));
    }

    #[test]
    fn test_remove_file_missing_is_ok() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        remove_file(&dir.path().join("absent.plist"))?;
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_write_file_creates_parent_then_remove() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("nested/autostart/pinel.desktop");
        write_file(&path, "x")?;
        assert_eq!(std::fs::read_to_string(&path)?, "x");
        remove_file(&path)?;
        assert!(!path.exists());
        Ok(())
    }
}
