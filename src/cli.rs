use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Validate the configuration file.
    Validate,

    /// Launch the specified application.
    Launch {
        #[arg(value_name = "APP_NAME")]
        app_name: String,
    },

    /// Show the specified application window.
    Show {
        #[arg(value_name = "APP_NAME")]
        app_name: String,
    },

    /// Hide the specified application window.
    Hide {
        #[arg(value_name = "APP_NAME")]
        app_name: String,
    },

    /// Focus the specified application window.
    Focus {
        #[arg(value_name = "APP_NAME")]
        app_name: String,
    },

    /// Toggle the specified application.
    Toggle {
        #[arg(value_name = "APP_NAME")]
        app_name: String,
    },
}

#[derive(Debug, Parser)]
#[command(about, long_about = None, version)]
pub struct Cli {
    /// Path to configuration file
    /// Default: `$XDG_CONFIG_HOME/niri/niri-app-hotkey.kdl`
    #[arg(short, long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}
