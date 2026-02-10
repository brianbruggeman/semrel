use std::path::PathBuf;

use package_json::PackageJson as PkgJson;
use serde_json::to_string;

use crate::{
    ManifestObjectSafe, ManifestStatic,
    core::{Manifest, ManifestError, SimpleVersion},
};

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct PackageJson {
    manifest: PkgJson,
}

impl PackageJson {
    pub fn new(version: impl Into<SimpleVersion>) -> Self {
        let version = version.into();
        let manifest = PkgJson {
            version: version.to_string(),
            ..Default::default()
        };
        Self { manifest }
    }
}

impl ManifestStatic for PackageJson {
    fn manifest_filename() -> &'static str {
        "package.json"
    }
}

impl ManifestObjectSafe for PackageJson {
    fn version(&self) -> Result<SimpleVersion, ManifestError> {
        let version = self
            .manifest
            .version
            .parse::<SimpleVersion>()
            .map_err(|e| ManifestError::InvalidManifest(format!("Invalid version part: {e}")))?;
        Ok(version)
    }

    fn set_version(&mut self, version: impl Into<SimpleVersion>) -> Result<(), ManifestError> {
        self.manifest.version = version.into().to_string();
        Ok(())
    }

    fn write(&self, path: impl Into<PathBuf>) -> Result<(), ManifestError> {
        let path = path.into();
        let data = serde_json::to_string_pretty(&self.manifest).map_err(|e| ManifestError::InvalidManifest(format!("Invalid manifest: {e}")))?;
        std::fs::write(path, data).map_err(|e| ManifestError::InvalidManifest(format!("Invalid manifest: {e}")))?;
        Ok(())
    }
}

impl Manifest for PackageJson {
    fn parse(data: impl AsRef<str>) -> Result<Self, ManifestError> {
        tracing::debug!("Parsing package.json");
        let manifest = serde_json::from_str::<PkgJson>(data.as_ref()).map_err(|e| ManifestError::InvalidManifest(format!("Invalid manifest: {e}")))?;
        tracing::trace!("Manifest: {manifest:?}");
        Ok(Self { manifest })
    }
}

impl PartialEq for PackageJson {
    fn eq(&self, other: &Self) -> bool {
        match (to_string(&self.manifest), to_string(&other.manifest)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ManifestError, SimpleVersion};
    use rstest::{fixture, rstest};
    use std::io::Write;
    use std::path::PathBuf;
    use std::{fs::File, str::FromStr};
    use tempfile::{TempDir, tempdir};

    #[fixture]
    fn temp_package_json() -> (TempDir, PathBuf, PathBuf) {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("package.json");

        let mut file = File::create(&file_path).unwrap();
        let data = r#"
        {
            "name": "test",
            "version": "1.0.0"
        }
        "#;
        write!(file, "{data}").unwrap();

        let parent = file_path.parent().unwrap().to_path_buf();
        (temp_dir, parent, file_path)
    }

    #[fixture]
    fn temp_invalid_package_version_json() -> (TempDir, PathBuf, PathBuf) {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("package.json");

        let mut file = File::create(&file_path).unwrap();
        let data = r#"
        {
            "name": "test",
            "version": "invalid"
        }
        "#;
        write!(file, "{data}").unwrap();

        let parent = file_path.parent().unwrap().to_path_buf();
        (temp_dir, parent, file_path)
    }

    #[fixture]
    fn temp_missing_package_json() -> (TempDir, PathBuf, PathBuf) {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("package.json");

        let parent = file_path.parent().unwrap().to_path_buf();
        (temp_dir, parent, file_path)
    }

    #[test]
    fn test_find_valid_package_json() {
        let (_temp_dir, parent, package_json_path) = temp_package_json();
        let result = PackageJson::find(&package_json_path);
        assert!(result.is_ok(), "Expected to find package.json, but got {:?}", result);
        assert_eq!(result.unwrap(), package_json_path);
        let result = PackageJson::find(parent);
        assert!(result.is_ok(), "Expected to find package.json, but got {:?}", result);
        assert_eq!(result.unwrap(), package_json_path);
    }

    #[test]
    fn test_load_valid_file() {
        let (_temp_dir, _parent, package_json_path) = temp_package_json();
        let result = PackageJson::load(package_json_path);
        let data = r#"
        {
            "name": "test",
            "version": "1.0.0"
        }
        "#;
        assert!(result.is_ok(), "Expected to load package.json, but got {:?}", result);
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_find_missing_package_json() {
        let (_temp_dir, parent, _) = temp_missing_package_json();
        let result = PackageJson::find(parent);
        assert!(result.is_err(), "Expected to not find package.json, but got {:?}", result);
    }

    #[rstest]
    #[case::validate_valid_json("{\"name\":\"test\",\"version\":\"1.0.0\"}", SimpleVersion::from_str("1.0.0").map_err(|_| ManifestError::InvalidManifest("Invalid version part: invalid digit found in string at line 1 column 42".to_string())))]
    #[case::validate_invalid_json("{\"name\":\"test\",\"version\":\"invalid-version\"}", SimpleVersion::from_str("invalid-version").map_err(|_| ManifestError::InvalidManifest("Invalid version part: Invalid version part: invalid digit found in string".to_string())))]
    fn test_parse(#[case] data: &str, #[case] expected: Result<SimpleVersion, crate::ManifestError>) {
        let result = PackageJson::parse(data);
        match &result {
            Ok(result) => assert_eq!(result.version(), expected),
            Err(result) => assert_eq!(result, &expected.unwrap_err()),
        }
    }

    #[rstest]
    #[case::parse_valid_version("{\"name\":\"test\",\"version\":\"1.0.0\"}", Ok(SimpleVersion::new(1, 0, 0)))]
    #[case::parse_invalid_version("{\"name\":\"test\",\"version\":\"invalid-version\"}", Err(ManifestError::InvalidManifest("Invalid version part: invalid digit found in string at line 1 column 42".to_string())))]
    #[case::parse_missing_version("{\"name\":\"test\"}", Err(ManifestError::InvalidManifest("missing field `version` at line 1 column 15".to_string())))]
    fn test_parse_version(#[case] data: &str, #[case] expected: Result<SimpleVersion, ManifestError>) {
        let result = PackageJson::parse_version(data);
        match (&result, &expected) {
            (Ok(result), Ok(expected)) => assert_eq!(result, expected),
            (Err(_result), Err(_expected)) => {}
            _ => panic!("{:?} result did not match expected {:?}", result, expected),
        }
    }
}
