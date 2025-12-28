use std::path::PathBuf;

use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use miette::{Result, miette};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Subcommand)]
pub enum Command {
    /// Validate the configuration file.
    Validate,

    /// Launch the specified application.
    Launch {
        #[arg(value_name = "APP_NAME")]
        application_name: String,
    },

    /// Show the specified application window.
    Show {
        #[arg(value_name = "APP_NAME")]
        application_name: String,
    },

    /// Hide the specified application window.
    Hide {
        #[arg(value_name = "APP_NAME")]
        application_name: String,
    },

    /// Activate the specified application window.
    Activate {
        #[arg(value_name = "APP_NAME")]
        application_name: String,
    },

    /// Toggle the specified application window.
    Toggle {
        #[arg(value_name = "APP_NAME")]
        application_name: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, Parser)]
#[command(about, long_about = None, version)]
struct CliInner {
    #[command(subcommand)]
    command: Command,

    /// Path to configuration file.
    /// Defaults to `$XDG_CONFIG_HOME/niri/niri-app-hotkey.kdl`.
    #[arg(
        short = 'c',
        long = "config",
        value_name = "PATH",
        verbatim_doc_comment
    )]
    config_path: Option<String>,
}

pub struct Cli {
    pub command: Command,
    pub config_path: PathBuf,
}

impl Cli {
    pub fn parse() -> Result<Self> {
        let cli = CliInner::parse();

        let command = cli.command;
        let config_path = cli
            .config_path
            .map(PathBuf::from)
            .ok_or(())
            .or_else(|_| Self::get_default_config_path())?;

        Ok(Self {
            command,
            config_path,
        })
    }

    fn get_default_config_path() -> Result<PathBuf> {
        let dirs = ProjectDirs::from_path(PathBuf::from("niri")).ok_or_else(|| {
            miette!("Could not determine default config path, please provide one via --config")
        })?;

        Ok(dirs.config_dir().join("niri-app-hotkey.kdl"))
    }
}
