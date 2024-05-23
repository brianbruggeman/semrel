use clap::ValueEnum;
use std::fmt;
use std::str::FromStr;

use crate::SimpleVersion;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BumpRule {
    /// Not set
    #[default]
    Notset,
    /// Explicitly do not bump the version
    NoBump,
    /// Bump the patch version
    Patch,
    /// Bump the minor version
    Minor,
    /// Bump the major version
    Major,
}

impl BumpRule {
    pub fn bump_version(&self, version: impl Into<SimpleVersion>) -> SimpleVersion {
        version.into().bump(*self)
    }
}

impl fmt::Display for BumpRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BumpRule::Major => write!(f, "major"),
            BumpRule::Minor => write!(f, "minor"),
            BumpRule::Patch => write!(f, "patch"),
            BumpRule::NoBump => write!(f, "none"),
            BumpRule::Notset => write!(f, "notset"),
        }
    }
}

impl FromStr for BumpRule {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "major" | "M" | "3" => Ok(BumpRule::Major),
            "minor" | "m" | "2" => Ok(BumpRule::Minor),
            "patch" | "p" | "y" | "yes" | "true" | "t" | "e" | "enable" | "on" | "1" => Ok(BumpRule::Patch),
            "none" | "n" | "no" | "false" | "f" | "d" | "disable" | "off" | "0" => Ok(BumpRule::NoBump),
            _ => Err(()),
        }
    }
}

impl From<&str> for BumpRule {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "major" | "M" | "3" => BumpRule::Major,
            "minor" | "m" | "2" => BumpRule::Minor,
            "patch" | "p" | "y" | "yes" | "true" | "t" | "e" | "enable" | "on" | "1" => BumpRule::Patch,
            "none" | "n" | "no" | "false" | "f" | "d" | "disable" | "off" | "0" => BumpRule::NoBump,
            _ => BumpRule::Notset,
        }
    }
}

impl ValueEnum for BumpRule {
    fn value_variants<'a>() -> &'a [Self] {
        &[BumpRule::Major, BumpRule::Minor, BumpRule::Patch, BumpRule::NoBump]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            BumpRule::Major => clap::builder::PossibleValue::new("major")
                .alias("+++")
                .alias("3")
                .alias("M")
                ,
            BumpRule::Minor => clap::builder::PossibleValue::new("minor")
                .alias("++")
                .alias("2")
                .alias("m")
                ,
            BumpRule::Patch => clap::builder::PossibleValue::new("patch")
                .alias("+")
                .alias("p")
                .alias("y")
                .alias("yes")
                .alias("true")
                .alias("t")
                .alias("e")
                .alias("enable")
                .alias("on")
                .alias("1")
                ,
            BumpRule::NoBump => clap::builder::PossibleValue::new("none")
                .alias("")
                .alias("-")
                .alias("n")
                .alias("no")
                .alias("false")
                .alias("f")
                .alias("d")
                .alias("disable")
                .alias("off")
                .alias("0")
                ,
            _ => clap::builder::PossibleValue::new("notset"),
        })
    }
}
