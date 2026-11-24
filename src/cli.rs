use std::path::PathBuf;

/// This file is responsible for providing strctures
/// for the following features:
/// - SSH connectivity
/// - CLI/TUI tool
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "pinel", version, about = "Pinel Editor")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    pub path: Option<PathBuf>,

    /// Opens the editor without restoring or saving the session.
    /// `pinel connect` passes this so editing a remote file
    /// does not replace the user's saved tabs.
    #[arg(long, hide = true)]
    pub no_session: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Connect {
        target: String,
        #[arg(default_value = ".")]
        path: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_no_session_flag() {
        let cli = Cli::parse_from(["pinel", "--no-session", "/tmp/file.rs"]);
        assert!(cli.no_session);
        assert_eq!(cli.path, Some(PathBuf::from("/tmp/file.rs")));
    }

    #[test]
    fn test_cli_parse_defaults_to_session() {
        let cli = Cli::parse_from(["pinel", "/tmp/file.rs"]);
        assert!(!cli.no_session);
    }
}
