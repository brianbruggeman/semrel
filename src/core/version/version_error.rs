#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum VersionError {
    #[error("Invalid version string: {0}")]
    InvalidVersionString(String),
    #[error("Invalid version part: {0}")]
    InvalidVersionPart(#[from] std::num::ParseIntError), // Automatically convert ParseIntError to VersionError
}
