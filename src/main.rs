use miette::Result;

mod action;
mod cli;
mod config;

fn main() -> Result<()> {
    let cli = cli::Cli::parse()?;
    let config = config::Config::parse(&cli.config_path)?;

    match cli.command {
        cli::Command::Validate => {
            println!("Configuration file is valid.");
        }
        cli::Command::Launch { application_name } => {
            let application = config.find_application(&application_name)?;
            action::launch(&application)?;
        }
        cli::Command::Show { application_name } => {
            let application = config.find_application(&application_name)?;
            action::show(&application)?;
        }
        cli::Command::Hide { application_name } => {
            let application = config.find_application(&application_name)?;
            action::hide(&application)?;
        }
        cli::Command::Activate { application_name } => {
            let application = config.find_application(&application_name)?;
            action::activate(&application)?;
        }
        cli::Command::Toggle { application_name } => {
            let application = config.find_application(&application_name)?;
            action::toggle(&application)?;
        }
    }

    Ok(())
}
