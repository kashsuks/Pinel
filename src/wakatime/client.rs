use std::process::{Command, Stdio};

use super::config::WakaTimeConfig;

const API_KEY_ENV: &str = "WAKATIME_API_KEY";

pub fn send_heartbeat(entity: &str, is_write: bool, cfg: &WakaTimeConfig) -> std::io::Result<()> {
    let Some(mut cmd) = build_command(entity, is_write, cfg) else {
        return Ok(());
    };

    cmd.stdout(Stdio::null()).stderr(Stdio::null());
    let _ = cmd.spawn()?;
    Ok(())
}

/// Builds the `wakatime-cli` invocation or `None` when no API key is set
fn build_command(entity: &str, is_write: bool, cfg: &WakaTimeConfig) -> Option<Command> {
    let api_key = cfg.api_key.trim();
    if api_key.is_empty() {
        return None;
    }

    let mut cmd = Command::new("wakatime-cli");
    cmd.arg("--entity").arg(entity);
    cmd.arg("--plugin").arg("pinel/0.1.0");
    cmd.env(API_KEY_ENV, api_key);

    if !cfg.api_url.trim().is_empty() {
        cmd.arg("--api-url").arg(cfg.api_url.trim());
    }
    if is_write {
        cmd.arg("--write");
    }
    Some(cmd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    fn config(api_key: &str, api_url: &str) -> WakaTimeConfig {
        WakaTimeConfig {
            api_key: api_key.to_string(),
            api_url: api_url.to_string(),
        }
    }

    fn args(cmd: &Command) -> Vec<String> {
        cmd.get_args().map(|a| a.to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn build_command_returns_none_without_an_api_key() {
        assert!(build_command("a.rs", false, &config("", "")).is_none());
        assert!(build_command("a.rs", false, &config("   ", "")).is_none());
    }

    #[test]
    fn build_command_never_puts_the_api_key_in_the_arguments() {
        let cmd = build_command("a.rs", true, &config("waka_secret", "https://x/api/v1")).unwrap();

        let args = args(&cmd);
        assert!(!args.iter().any(|a| a.contains("waka_secret")));
        assert!(!args.iter().any(|a| a == "--key"));
    }

    #[test]
    fn build_command_passes_the_trimmed_api_key_through_the_environment() {
        let cmd = build_command("a.rs", false, &config("  waka_secret\n", "")).unwrap();

        let value = cmd
            .get_envs()
            .find(|(name, _)| *name == OsStr::new(API_KEY_ENV))
            .and_then(|(_, value)| value);
        assert_eq!(value, Some(OsStr::new("waka_secret")));
    }

    #[test]
    fn build_command_keeps_entity_write_flag_and_api_url() {
        let cmd =
            build_command("src/a.rs", true, &config("k", "https://hackatime/api/v1")).unwrap();

        assert_eq!(
            args(&cmd),
            [
                "--entity",
                "src/a.rs",
                "--plugin",
                "pinel/0.1.0",
                "--api-url",
                "https://hackatime/api/v1",
                "--write"
            ]
        );
    }

    #[test]
    fn build_command_omits_write_flag_and_empty_api_url() {
        let cmd = build_command("a.rs", false, &config("k", "  ")).unwrap();

        assert_eq!(args(&cmd), ["--entity", "a.rs", "--plugin", "pinel/0.1.0"]);
    }

    #[test]
    fn build_command_sends_write_flag_without_a_custom_api_url() {
        let cmd = build_command("a.rs", true, &config("k", "  ")).unwrap();

        assert_eq!(
            args(&cmd),
            ["--entity", "a.rs", "--plugin", "pinel/0.1.0", "--write"]
        );
    }

    #[test]
    fn build_command_omits_write_flag_with_a_custom_api_url() {
        let cmd = build_command("a.rs", false, &config("k", "https://hackatime/api/v1")).unwrap();

        assert_eq!(
            args(&cmd),
            [
                "--entity",
                "a.rs",
                "--plugin",
                "pinel/0.1.0",
                "--api-url",
                "https://hackatime/api/v1"
            ]
        );
    }
}
