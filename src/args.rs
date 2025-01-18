use std::path::PathBuf;

use clap::Parser;

use crate::PROJECT_DIRS;

/// tmux workspace manager
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Arguments {
    /// Config file location
    #[arg(short, long, env = "TMUXY_CONFIG", default_value = default_config_path().into_os_string())]
    pub config: PathBuf,
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
