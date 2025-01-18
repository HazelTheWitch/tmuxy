use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::PROJECT_DIRS;

/// tmux workspace manager
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    /// Config file location
    #[arg(short, long, env = "TMUXY_CONFIG", default_value = default_config_path().into_os_string())]
    pub config: PathBuf,
    /// Only print commands and do not run anything
    #[arg(short, long)]
    pub dry_run: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Open a workspace
    #[command(alias = "o")]
    Open(OpenArguments),
    /// Close a workspace
    #[command(alias = "c")]
    Close {
        /// Workspace to close
        #[arg(default_value = "default")]
        workspace: String,
    },
    #[command(alias = "u")]
    Update,
}

#[derive(Args)]
pub struct OpenArguments {
    /// Workspace to load
    #[arg(default_value = "default")]
    pub workspace: String,
    /// Working directory to open workspace in
    pub working_directory: Option<PathBuf>,
    /// Recreate workspace if already exists
    #[arg(short, long)]
    pub recreate: bool,
}

fn default_config_path() -> PathBuf {
    PROJECT_DIRS.config_dir().join("config.toml")
}
