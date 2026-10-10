use std::ops::Deref;
use std::str::{self, FromStr};

use miette::Result;
use regex::Regex as OriginalRegex;

#[derive(Clone, Debug)]
pub struct Regex(pub OriginalRegex);

impl FromStr for Regex {
    type Err = <OriginalRegex as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        OriginalRegex::from_str(s).map(Self)
    }
}

impl Deref for Regex {
    type Target = OriginalRegex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
