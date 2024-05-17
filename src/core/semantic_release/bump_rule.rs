use std::fmt;

use crate::SimpleVersion;

pub enum BumpRule {
    Major,
    Minor,
    Patch,
    NoBump,
}

impl BumpRule {
    pub fn bump_version(&self, version: impl Into<SimpleVersion>) -> SimpleVersion {
        let mut version = version.into();
        match self {
            BumpRule::Major => version.increment_major(),
            BumpRule::Minor => version.increment_minor(),
            BumpRule::Patch => version.increment_patch(),
            _ => {}
        }
        version
    }
}

impl fmt::Display for BumpRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BumpRule::Major => write!(f, "major"),
            BumpRule::Minor => write!(f, "minor"),
            BumpRule::Patch => write!(f, "patch"),
            BumpRule::NoBump => write!(f, "no bump"),
        }
    }
}
