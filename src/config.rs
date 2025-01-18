use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

bounded_integer::bounded_integer! {
    pub struct Percent { 0..=100 }
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(rename = "workspace")]
    pub workspaces: HashMap<String, Workspace>,
}

#[derive(Deserialize)]
pub struct Workspace {
    pub windows: Vec<Window>,
}

#[derive(Deserialize)]
pub struct Window {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub pane: Pane,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Pane {
    Leaf {
        #[serde(default)]
        command: Option<String>,
        #[serde(default)]
        directory: Option<PathBuf>,
    },
    Split {
        first: Box<Self>,
        second: Box<Self>,
        direction: Direction,
        #[serde(default = "pane_default_percent")]
        percent: Percent,
    },
}

impl Default for Pane {
    fn default() -> Self {
        Self::Leaf {
            command: None,
            directory: None,
        }
    }
}

impl Pane {
    pub fn visit_pane<'p>(
        &'p self,
        visit_split: &mut impl FnMut(Direction, Percent) -> color_eyre::Result<()>,
        visit_pane: &mut impl FnMut(Option<&'p str>, Option<&'p Path>) -> color_eyre::Result<()>,
    ) -> color_eyre::Result<()> {
        match self {
            Pane::Split {
                first,
                second,
                direction,
                percent,
            } => {
                first.visit_pane(visit_split, visit_pane)?;

                visit_split(*direction, *percent)?;

                second.visit_pane(visit_split, visit_pane)?;

                Ok(())
            }
            Pane::Leaf { command, directory } => {
                visit_pane(command.as_deref(), directory.as_deref())
            }
        }
    }
}

fn pane_default_percent() -> Percent {
    Percent::new(50).unwrap()
}

#[derive(Clone, Copy, Deserialize)]
pub enum Direction {
    Vertical,
    Horizontal,
}

pub fn parse_config(path: impl AsRef<Path>) -> color_eyre::Result<Config> {
    Ok(toml::from_str(&fs::read_to_string(path)?)?)
}
