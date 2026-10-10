use std::path::{Path, PathBuf};
use std::str::{self};

use knus::Decode;
use miette::{Context, IntoDiagnostic, Result, bail};

mod app;
mod regex;
mod rule;

pub use self::app::App;

#[derive(Clone, Debug, Decode)]
pub struct Config {
    #[knus(children(name = "application"))]
    pub apps: Vec<App>,
}

impl Config {
    pub fn from(arg: Option<&Path>) -> Result<Self> {
        if let Some(config) = Self::from_arg_path(arg)? {
            return Ok(config);
        }

        if let Some(config) = Self::from_default_path()? {
            return Ok(config);
        }

        bail!("No configuration file found");
    }

    fn from_arg_path(path: Option<&Path>) -> Result<Option<Self>> {
        let path = match path {
            Some(path) if Self::check_path(path) => path,
            Some(path) => {
                bail!("Config file does not exist at: {path:?}");
            }
            None => return Ok(None),
        };

        Self::load_from_path(path).map(Some)
    }

    fn from_default_path() -> Result<Option<Self>> {
        let path = directories::ProjectDirs::from_path(PathBuf::from("niri"))
            .map(|dir| dir.config_dir().join("niri-app-hotkey.kdl"));

        let path = match path.as_deref() {
            Some(path) if Self::check_path(path) => path,
            Some(_) => return Ok(None),
            None => return Ok(None),
        };

        Self::load_from_path(path).map(Some)
    }

    fn check_path(path: &Path) -> bool {
        path.exists() && path.is_file()
    }

    fn load_from_path(path: &Path) -> Result<Self> {
        let file_path = path.to_string_lossy().to_string();
        let content = std::fs::read_to_string(path)
            .into_diagnostic()
            .context(format!("Failed to read config file at: {file_path}"))?;

        knus::parse(&file_path, &content)
            .context(format!("Failed to parse config file at: {file_path}"))
    }

    pub fn find_app_by_name(&self, name: &str) -> Result<&App> {
        let Some(app) = self.apps.iter().find(|app| app.name == name) else {
            bail!("Application with name '{name}' not found in configuration");
        };

        Ok(app)
    }
}
