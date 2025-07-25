use std::{
    fmt::{self, Display},
    str::FromStr,
};

use num_traits::AsPrimitive;
use serde::Deserialize;
use serde::de::{self, Deserializer, Visitor};

use super::{Ver, VersionError};
use crate::BumpRule;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash, serde::Serialize)]
pub struct SimpleVersion {
    major: Ver,
    minor: Ver,
    patch: Ver,
}

impl SimpleVersion {
    pub fn new(major: impl AsPrimitive<Ver>, minor: impl AsPrimitive<Ver>, patch: impl AsPrimitive<Ver>) -> Self {
        SimpleVersion {
            major: major.as_(),
            minor: minor.as_(),
            patch: patch.as_(),
        }
    }

    pub fn increment_major(&mut self) {
        self.major += 1;
        self.minor = 0;
        self.patch = 0;
    }

    pub fn increment_minor(&mut self) {
        self.minor += 1;
        self.patch = 0;
    }

    pub fn increment_patch(&mut self) {
        self.patch += 1;
    }

    pub fn major(&self) -> Ver {
        self.major
    }

    pub fn minor(&self) -> Ver {
        self.minor
    }

    pub fn patch(&self) -> Ver {
        self.patch
    }

    pub fn bump(&self, rule: impl Into<BumpRule>) -> SimpleVersion {
        match rule.into() {
            BumpRule::Major => {
                let mut new_version = *self;
                new_version.increment_major();
                new_version
            }
            BumpRule::Minor => {
                let mut new_version = *self;
                new_version.increment_minor();
                new_version
            }
            BumpRule::Patch => {
                let mut new_version = *self;
                new_version.increment_patch();
                new_version
            }
            BumpRule::NoBump | BumpRule::Notset => *self,
        }
    }
}

impl<'de> Deserialize<'de> for SimpleVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VersionVisitor;

        impl<'de> Visitor<'de> for VersionVisitor {
            type Value = SimpleVersion;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a version string in the format 'major.minor.patch'")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                SimpleVersion::from_str(v).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(VersionVisitor)
    }
}

impl std::ops::Add<BumpRule> for SimpleVersion {
    type Output = Self;

    fn add(self, rule: BumpRule) -> Self::Output {
        self.bump(rule)
    }
}

impl<S> PartialEq<S> for SimpleVersion
where
    S: AsRef<str>,
{
    fn eq(&self, other: &S) -> bool {
        match SimpleVersion::from_str(other.as_ref()) {
            Ok(v) => &v == self,
            Err(_why) => false,
        }
    }
}

impl std::ops::Add<SimpleVersion> for BumpRule {
    type Output = SimpleVersion;

    fn add(self, version: SimpleVersion) -> Self::Output {
        version.bump(self)
    }
}

impl From<&str> for SimpleVersion {
    fn from(value: &str) -> Self {
        value.parse().unwrap_or_default()
    }
}

impl FromStr for SimpleVersion {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut version = SimpleVersion::default();
        let parts: Vec<&str> = s.split('.').collect();
        // If parsing fails, ParseIntError is automatically converted to VersionError::InvalidVersionPart
        match parts.len() {
            3 => {
                version.major = parts[0].parse()?;
                version.minor = parts[1].parse()?;
                version.patch = parts[2].parse()?;
            }
            2 => {
                version.major = parts[0].parse()?;
                version.minor = parts[1].parse()?;
            }
            1 => {
                version.major = parts[0].parse()?;
            }
            _ => return Err(VersionError::InvalidVersionString(s.to_string())),
        }

        Ok(version)
    }
}

impl Display for SimpleVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::v1_0_0("1.0.0", "1.0.0")]
    #[case::v1_0("1.0", "1.0.0")]
    #[case::v1("1", "1.0.0")]
    #[case::v0_1_0("0.1.0", "0.1.0")]
    #[case::v1_2_3("1.2.3", "1.2.3")]
    #[case::v10_20_30("10.20.30", "10.20.30")]
    fn test_version_from_str(#[case] input: &str, #[case] expected: impl AsRef<str>) {
        let version: SimpleVersion = input.parse().unwrap();
        assert_eq!(version.to_string(), expected.as_ref());
    }

    #[rstest]
    #[case::invalid_version_too_long("1.2.3.4")]
    #[case::invalid_version_no_numerics("a.b.c")]
    #[case::invalid_version_not_enough_numerics("1.2.c")]
    fn test_version_from_str_invalid(#[case] input: &str) {
        let version: Result<SimpleVersion, VersionError> = input.parse();
        assert!(version.is_err());
    }

    #[rstest]
    #[case::v1_2_3(1, 2, 3, "1.2.3")]
    fn test_version_display(#[case] major: impl AsPrimitive<Ver>, #[case] minor: impl AsPrimitive<Ver>, #[case] patch: impl AsPrimitive<Ver>, #[case] expected: impl AsRef<str>) {
        let version = SimpleVersion::new(major, minor, patch);
        assert_eq!(version.major(), major.as_());
        assert_eq!(version.minor(), minor.as_());
        assert_eq!(version.patch(), patch.as_());
        assert_eq!(version.to_string(), expected.as_ref());
    }
}
