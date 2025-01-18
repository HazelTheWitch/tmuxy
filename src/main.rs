use std::{
    env::current_dir,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};

use axoupdater::AxoUpdater;
use clap::Parser;
use color_eyre::eyre::bail;
use tmuxy::{
    args::{self, Arguments, OpenArguments},
    config::{parse_config, Config, Direction, Percent},
};

lazy_static::lazy_static! {
    pub static ref WORKING_DIR: PathBuf = current_dir().unwrap();
}

fn tmux_command_status(
    args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> color_eyre::Result<()> {
    let status = Command::new("tmux")
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if !status.success() {
        bail!("Failed executing command: {status}");
    }

    Ok(())
}

fn normalize_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_owned();
    }

    WORKING_DIR.join(path)
}

fn tmux_respawn_pane(
    session: &str,
    window: usize,
    pane: usize,
    command: Option<&str>,
    starting_directory: Option<&Path>,
) -> color_eyre::Result<()> {
    let pane_id_str = format!("{session}:{window}.{pane}");
    let pane_id = OsStr::new(&pane_id_str);

    let starting_directory = starting_directory.map(normalize_path);

    let args = [
        OsStr::new("respawn-pane"),
        OsStr::new("-k"),
        OsStr::new("-t"),
        pane_id,
    ]
    .into_iter()
    .chain(
        starting_directory
            .iter()
            .flat_map(|dir| [OsStr::new("-c"), dir.as_os_str()]),
    );

    tmux_command_status(args)?;

    if let Some(command) = command {
        tmux_command_status(["send-keys", "-t", &pane_id_str, "-l", command])?;
    }

    Ok(())
}

fn tmux_attach(session: &str) -> color_eyre::Result<ExitCode> {
    let status = Command::new("tmux")
        .arg("attach-session")
        .arg("-t")
        .arg(session)
        .status()?;

    if status.success() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}

fn tmux_kill(session: &str) -> color_eyre::Result<ExitCode> {
    let status = Command::new("tmux")
        .arg("kill-session")
        .arg("-t")
        .arg(session)
        .status()?;

    if status.success() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}

fn open(arguments: &OpenArguments, config: Config) -> color_eyre::Result<ExitCode> {
    let Some(workspace) = config.workspaces.get(&arguments.workspace) else {
        bail!("Workspace {} does not exist", &arguments.workspace);
    };

    if workspace.windows.is_empty() {
        bail!("Workspace {} has no windows", &arguments.workspace);
    }

    let session_name = &arguments.workspace;

    let session_exists = tmux_command_status(["has-session", "-t", session_name]).is_ok();

    if session_exists {
        if arguments.recreate {
            tmux_command_status(["kill-session", "-t", session_name])?;
        } else {
            return tmux_attach(session_name);
        }
    }

    let mut first = true;

    for (i, window) in workspace.windows.iter().enumerate() {
        let window_id = format!("{session_name}:{i}");
        let window_name_args = window.name.iter().flat_map(|name| ["-n", name]);

        if first {
            tmux_command_status(
                ["new-session", "-s", session_name, "-d"]
                    .into_iter()
                    .chain(window_name_args),
            )?;
            first = false;
        } else {
            tmux_command_status(
                ["new-window", "-d", "-t", &window_id]
                    .into_iter()
                    .chain(window_name_args),
            )?;
        }

        let current_index = AtomicUsize::new(0);

        let mut visit_split = |direction, percent: Percent| {
            let flag = match direction {
                Direction::Vertical => "-v",
                Direction::Horizontal => "-h",
            };

            let current_index = current_index.fetch_add(1, Ordering::Relaxed);

            tmux_command_status([
                "split-window",
                "-t",
                &format!("{session_name}:{i}.{current_index}"),
                flag,
                "-p",
                &percent.to_string(),
            ])?;

            Ok(())
        };
        let mut visit_pane = |command, working_directory| {
            tmux_respawn_pane(
                session_name,
                i,
                current_index.load(Ordering::Relaxed),
                command,
                working_directory,
            )
        };

        window.pane.visit_pane(&mut visit_split, &mut visit_pane)?;
    }

    tmux_attach(session_name)
}

fn update() -> color_eyre::Result<ExitCode> {
    if let Some(result) = AxoUpdater::new_for("tmuxy").load_receipt()?.run_sync()? {
        let version_note = if let Some(old) = result.old_version {
            format!("{old} -> {}", result.new_version)
        } else {
            format!("{}", result.new_version)
        };

        eprintln!("tmuxy has been successfully updated ({version_note})");
    } else {
        eprintln!("tmuxy is already up to date");
    }

    Ok(ExitCode::SUCCESS)
}

fn main() -> color_eyre::Result<ExitCode> {
    color_eyre::install()?;

    let arguments = Arguments::parse();

    if !fs::exists(&arguments.config)? {
        if let Some(parent) = arguments.config.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&arguments.config, include_bytes!("../default_config.toml"))?;
    }

    if fs::metadata(&arguments.config)?.is_dir() {
        bail!("Config file at {:?} is a directory.", arguments.config);
    }

    let config = parse_config(&arguments.config)?;

    let result = match &arguments.command {
        args::Command::Open(open_arguments) => open(open_arguments, config),
        args::Command::Close { workspace } => tmux_kill(workspace),
        args::Command::Update => update(),
    };

    if !matches!(arguments.command, args::Command::Update)
        && AxoUpdater::new_for("tmuxy")
            .load_receipt()?
            .is_update_needed_sync()?
    {
        eprintln!("new update found for tmuxy, consider running `tmuxy update` to update");
    }

    result
}
