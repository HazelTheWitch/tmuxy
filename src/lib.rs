use directories::ProjectDirs;

pub mod args;
pub mod config;

lazy_static::lazy_static! {
    pub static ref PROJECT_DIRS: ProjectDirs = ProjectDirs::from("dev.setaria", "setaria", "tmuxy").unwrap();
}
