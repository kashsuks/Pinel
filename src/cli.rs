/// This file is responsible for providing strctures
/// for the following features:
/// - SSH connectivity
/// - CLI/TUI tool
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "pinel", version, about = "Pinel Editor")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    pub path: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Connect {
        target: String,
        #[arg(default_value = ".")]
        path: String,
    },
}
