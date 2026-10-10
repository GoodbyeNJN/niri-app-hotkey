use clap::Parser;
use miette::{Result, miette};

mod actions;
mod cli;
mod config;
mod niri;

use actions::*;
use cli::{Cli, Command};
pub(crate) use config::{App, Config};
pub(crate) use niri::Niri;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Command::Validate => {
            let _ = Config::from(cli.config.as_deref())?;
            println!("Configuration file is valid.");

            Ok(())
        }

        Command::Launch { app_name }
        | Command::Show { app_name }
        | Command::Hide { app_name }
        | Command::Focus { app_name }
        | Command::Toggle { app_name } => {
            let config = Config::from(cli.config.as_deref())?;
            let app = config.find_app_by_name(&app_name)?;

            match &cli.command {
                Command::Launch { .. } => launch(app),
                Command::Show { .. } => show(app),
                Command::Hide { .. } => hide(app),
                Command::Focus { .. } => focus(app),
                Command::Toggle { .. } => toggle(app),
                _ => unreachable!(),
            }
            .map_err(|e| miette!("{e:?}"))
        }
    }
}
