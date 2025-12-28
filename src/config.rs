use std::{
    fs,
    path::PathBuf,
    str::{self, FromStr},
};

use knus::Decode;
use miette::{Context, IntoDiagnostic, Result, miette};
use regex::Regex as OriginalRegex;

#[derive(Clone, Debug)]
pub struct Regex(pub OriginalRegex);
impl FromStr for Regex {
    type Err = <OriginalRegex as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        OriginalRegex::from_str(s).map(Self)
    }
}

#[derive(Clone, Debug, Decode)]
pub struct MatchRule {
    #[knus(property, str)]
    pub app_id: Option<Regex>,
    #[knus(property, str)]
    pub title: Option<Regex>,
    #[knus(property)]
    pub index: Option<usize>,
}

#[derive(Clone, Debug, Decode)]
pub struct Application {
    #[knus(argument)]
    pub name: String,
    #[knus(child, unwrap(arguments))]
    pub spawn: Option<Vec<String>>,
    #[knus(child, unwrap(argument))]
    pub spawn_sh: Option<String>,
    #[knus(children(name = "match"))]
    pub matches: Vec<MatchRule>,
    #[knus(children(name = "exclude"))]
    pub excludes: Vec<MatchRule>,
}

#[derive(Clone, Debug, Decode)]
pub struct Config {
    #[knus(children(name = "application"))]
    pub applications: Vec<Application>,
}

impl Config {
    pub fn parse(path: &PathBuf) -> Result<Self> {
        let file_name = path
            .as_os_str()
            .to_str()
            .ok_or_else(|| miette!("Invalid config file name"))?;
        let text = fs::read_to_string(path)
            .into_diagnostic()
            .context(format!("Failed to read config file at: {path:?}"))?;

        knus::parse(file_name, &text).context(format!("Failed to parse config file at: {path:?}"))
    }

    pub fn find_application(&self, name: &str) -> Result<&Application> {
        self.applications
            .iter()
            .find(|app| app.name == name)
            .ok_or_else(|| {
                miette!(
                    "Application with name '{}' not found in configuration.",
                    name
                )
            })
    }
}
