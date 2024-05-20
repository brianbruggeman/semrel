use clap::ValueEnum;
use std::fmt;
use std::str::FromStr;

use crate::SimpleVersion;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BumpRule {
    /// Bump the major version
    Major,
    /// Bump the minor version
    Minor,
    /// Bump the patch version
    Patch,
    /// Explicitly do not bump the version
    NoBump,
    /// Not set
    #[default]
    Notset,
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
        match s {
            "major" => Ok(BumpRule::Major),
            "minor" => Ok(BumpRule::Minor),
            "patch" => Ok(BumpRule::Patch),
            "none" => Ok(BumpRule::NoBump),
            _ => Err(()),
        }
    }
}

impl From<&str> for BumpRule {
    fn from(s: &str) -> Self {
        match s {
            "major" => BumpRule::Major,
            "minor" => BumpRule::Minor,
            "patch" => BumpRule::Patch,
            "none" => BumpRule::NoBump,
            _ => BumpRule::Notset,
        }
    }
}

impl ValueEnum for BumpRule {
    fn value_variants<'a>() -> &'a [Self] {
        &[BumpRule::Major, BumpRule::Minor, BumpRule::Patch, BumpRule::NoBump, BumpRule::Notset]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            BumpRule::Major => clap::builder::PossibleValue::new("major"),
            BumpRule::Minor => clap::builder::PossibleValue::new("minor"),
            BumpRule::Patch => clap::builder::PossibleValue::new("patch"),
            BumpRule::NoBump => clap::builder::PossibleValue::new("none"),
            BumpRule::Notset => clap::builder::PossibleValue::new("notset"),
        })
    }
}
