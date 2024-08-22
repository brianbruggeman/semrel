use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::SimpleVersion;

use crate::{
    core::{Manifest, ManifestError},
    ManifestObjectSafe, ManifestStatic,
};

#[derive(Debug, serde::Deserialize, PartialEq, Clone)]
pub struct CargoToml {
    manifest: cargo_toml::Manifest,
}

impl CargoToml {
    pub fn new(version: impl Into<SimpleVersion>) -> Self {
        let version = version.into();
        let version_string = version.to_string();
        let data = format!(
            r#"
                [package]
                name = "default"
                version = "{}"
            "#,
            version_string
        );
        let manifest = cargo_toml::Manifest::from_slice(data.as_bytes()).expect("Failed to parse default Cargo.toml");
        Self { manifest }
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ManifestError> {
        let data = std::fs::read_to_string(path).expect("Failed to read file");
        Self::from_str(&data)
    }
}

impl FromStr for CargoToml {
    type Err = ManifestError;

    fn from_str(data: &str) -> Result<Self, Self::Err> {
        let data = data.as_bytes();
        let manifest = cargo_toml::Manifest::from_slice(data).map_err(|why| ManifestError::InvalidManifest(why.to_string()))?;
        Ok(Self { manifest })
    }
}

impl From<PathBuf> for CargoToml {
    fn from(path: PathBuf) -> Self {
        Self::from_path(path).expect("Failed to parse Cargo.toml")
    }
}

impl From<&Path> for CargoToml {
    fn from(path: &Path) -> Self {
        Self::from_path(path).expect("Failed to parse Cargo.toml")
    }
}

impl Default for CargoToml {
    fn default() -> Self {
        let default_cargo_toml = r#"
            [package]
            name = "default"
            version = "0.1.0"
        "#
        .as_bytes();
        let manifest = cargo_toml::Manifest::from_slice(default_cargo_toml).expect("Failed to parse default Cargo.toml");
        Self { manifest }
    }
}

impl ManifestStatic for CargoToml {
    fn manifest_filename() -> &'static str {
        "Cargo.toml"
    }
}

impl ManifestObjectSafe for CargoToml {
    fn version(&self) -> Result<SimpleVersion, ManifestError> {
        match &self.manifest.package {
            Some(package) => match package.version.get() {
                Ok(version) => {
                    if version == "1.0.0" {
                        tracing::trace!("package: {:?}", package);
                    }
                    SimpleVersion::from_str(version.as_ref())
                        .map_err(|why| ManifestError::InvalidManifest(why.to_string()))
                        .map(|version| match version == SimpleVersion::new(0, 0, 0) {
                            true => Err(ManifestError::InvalidManifest("Invalid version".to_string())),
                            false => Ok(version),
                        })?
                }
                Err(why) => Err(ManifestError::InvalidManifest(why.to_string())),
            },
            None => Err(ManifestError::InvalidManifest("Missing package".to_string())),
        }
    }

    fn set_version(&mut self, version: impl Into<SimpleVersion>) -> Result<(), ManifestError> {
        let version = version.into();
        let version_string = version.to_string();
        if let Some(package) = self.manifest.package.as_mut() {
            package.version.set(version_string);
        }
        Ok(())
    }

    fn write(&self, path: impl Into<PathBuf>) -> Result<(), ManifestError> {
        let toml_string = toml::to_string(&self.manifest).map_err(|why| ManifestError::InvalidManifest(why.to_string()))?;
        let mut file = File::create(path.into()).map_err(|why| ManifestError::InvalidManifest(why.to_string()))?;
        file.write_all(toml_string.as_bytes())
            .map_err(|why| ManifestError::InvalidManifest(why.to_string()))?;
        Ok(())
    }
}

impl Manifest for CargoToml {
    fn parse(data: impl AsRef<str>) -> Result<Self, ManifestError> {
        tracing::trace!("Parsing Cargo.toml");
        let data = data.as_ref().as_bytes();
        if data.is_empty() {
            return Err(ManifestError::InvalidManifest("Manifest is empty!".to_string()));
        }
        let manifest = cargo_toml::Manifest::from_slice(data).map_err(|why| ManifestError::InvalidManifest(why.to_string()))?;
        tracing::trace!("Parsed manifest.");
        Ok(Self { manifest })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    use crate::core::{ManifestError, SimpleVersion};
    use rstest::{fixture, rstest};
    use tempfile::{tempdir, TempDir};

    #[fixture]
    fn temp_cargo_toml() -> (TempDir, PathBuf, PathBuf) {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("Cargo.toml");

        // Create a Cargo.toml file with valid content
        let mut file = File::create(&file_path).unwrap();
        let data = r#"
            [package]
            name = "test"
            version = "1.0.0"
        "#;
        write!(file, "{data}").unwrap();

        let parent = file_path.parent().unwrap().to_path_buf();
        (temp_dir, parent, file_path)
    }

    #[fixture]
    fn temp_invalid_package_version_json() -> (TempDir, PathBuf, PathBuf) {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("Cargo.toml");

        // Create a Cargo.toml file with invalid version
        let mut file = File::create(&file_path).unwrap();
        let data = r#"
            [package]
            name = "test"
            version = "invalid"
        "#;
        write!(file, "{data}").unwrap();

        let parent = file_path.parent().unwrap().to_path_buf();
        (temp_dir, parent, file_path)
    }
    #[fixture]
    fn temp_missing_cargo_toml() -> (TempDir, PathBuf, PathBuf) {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("Cargo.toml");

        let parent = file_path.parent().unwrap().to_path_buf();
        (temp_dir, parent, file_path)
    }

    #[test]
    fn test_find_valid_cargo_toml() {
        let (_temp_dir, parent, cargo_toml_path) = temp_cargo_toml();
        let result = CargoToml::find(&cargo_toml_path);
        assert!(result.is_ok(), "Expected to find Cargo.toml, but got {:?}", result);
        assert_eq!(result.unwrap(), cargo_toml_path);
        let result = CargoToml::find(parent);
        assert!(result.is_ok(), "Expected to find Cargo.toml, but got {:?}", result);
        assert_eq!(result.unwrap(), cargo_toml_path);
    }

    #[test]
    fn test_load_valid_file() {
        let (_temp_dir, _parent, cargo_toml_path) = temp_cargo_toml();
        let result = CargoToml::load(cargo_toml_path);
        let data = r#"
            [package]
            name = "test"
            version = "1.0.0"
        "#;
        assert!(result.is_ok(), "Expected to load Cargo.toml, but got {:?}", result);
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_find_missing_cargo_toml() {
        let (_temp_dir, parent, _) = temp_missing_cargo_toml();
        let result = CargoToml::find(parent);
        assert!(result.is_err(), "Expected to not find Cargo.toml, but got {:?}", result);
    }

    #[rstest]
    #[case::validate_valid_version("[package]\nname = \"test\"\nversion = \"1.0.0\"\n", Ok(CargoToml::new("1.0.0")))]
    #[case::validate_invalid_version("[package]\nname = \"test\"\nversion = \"invalid-version\"\n", Err(ManifestError::InvalidManifest("Invalid manifest: Invalid version part: invalid digit found in string at line 1 column 37".to_string())))]
    #[case::parse_missing_version("[package]\nname = \"test\"\n", Err(ManifestError::InvalidManifest("TOML parse error at line 1, column 1\n  |\n1 | [package]\n  | ^^^^^^^^^\nmissing field `version`\n".to_string())))]
    fn test_parse(#[case] data: &str, #[case] expected: Result<CargoToml, ManifestError>) {
        let result = CargoToml::parse(data);
        match (&result, expected.as_ref()) {
            (Ok(result), Ok(expected_toml)) => match (&result.version(), expected_toml.version()) {
                (Ok(result_version), Ok(expected_version)) => assert_eq!(*result_version, expected_version, "Expected {expected:?} but got {result:?}"),
                (Err(_result), Err(_expected)) => {}
                _ => panic!("Expected {expected:?} but got {result:?}"),
            },
            (Err(_result), Err(_expected)) => {}
            (Ok(result), Err(_expected_version_error)) => match result.version() {
                Err(_result_version_error) => {}
                _ => panic!("Expected {expected:?} but got {result:?}"),
            },
            _ => panic!("Expected {expected:?} but got {result:?}"),
        }
    }

    #[rstest]
    #[case::parse_valid_version("[package]\nname = \"test\"\nversion = \"1.0.0\"\n", Ok(SimpleVersion::new(1, 0, 0)))]
    #[case::parse_invalid_version("[package]\nname = \"test\"\nversion = \"invalid-version\"\n", Err(ManifestError::InvalidManifest("TOML parse error at line 3, column 11\n  |\n3 | version = \"invalid-version\"\n  |           ^^^^^^^^^^^^^^^^^\nInvalid version part: invalid digit found in string\n".to_string())))]
    #[case::parse_missing_version("[package]\nname = \"test\"\n", Err(ManifestError::InvalidManifest("TOML parse error at line 1, column 1\n  |\n1 | [package]\n  | ^^^^^^^^^\nmissing field `version`\n".to_string())))]
    fn test_parse_version(#[case] data: &str, #[case] expected: Result<SimpleVersion, ManifestError>) {
        let result = CargoToml::parse(data).expect("Failed to parse Cargo.toml");
        match (&result.version(), &expected.as_ref()) {
            (Ok(result), Ok(expected)) => assert_eq!(result, *expected, "Expected {expected:?} but got {result:?}"),
            // For the moment, don't worry about precise parsing errors
            (Err(_result), Err(_expected)) => {}
            (Ok(result), Err(_)) => panic!("\n\nresult: {result:?}\nresult did not match expected\nExpected: {expected:?}\n\n"),
            _ => panic!("\n\nresult: {result:?}\nresult did not match expected\nExpected: {expected:?}\n\n"),
        }
    }
}
