use crate::{
    core::{Manifest, ManifestError, SimpleVersion},
    ManifestObjectSafe, ManifestStatic,
};
#[derive(Debug, Default, serde::Deserialize, PartialEq, Eq, Clone, Copy)]
pub struct PackageJson {
    pub version: SimpleVersion,
}

impl PackageJson {
    pub fn new(version: impl Into<SimpleVersion>) -> Self {
        Self { version: version.into() }
    }
}

impl ManifestStatic for PackageJson {
    fn manifest_filename() -> &'static str {
        "package.json"
    }
}

impl ManifestObjectSafe for PackageJson {
    fn version(&self) -> Result<SimpleVersion, ManifestError> {
        Ok(self.version)
    }
}

impl Manifest for PackageJson {
    fn parse(data: impl AsRef<str>) -> Result<Self, ManifestError> {
        let package: Self = serde_json::from_str(data.as_ref()).map_err(|why| ManifestError::InvalidManifest(why.to_string()))?;
        Ok(package)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ManifestError, SimpleVersion};
    use rstest::{fixture, rstest};
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::{tempdir, TempDir};

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
    #[case::validate_valid_json("{\"name\":\"test\",\"version\":\"1.0.0\"}", Ok(PackageJson::new("1.0.0")))]
    #[case::validate_invalid_json("{\"name\":\"test\",\"version\":\"invalid-version\"}", Err(ManifestError::InvalidManifest("Invalid manifest: invalid digit found in string at line 1 column 31".to_string())))]
    fn test_parse(#[case] data: &str, #[case] expected: Result<PackageJson, ManifestError>) {
        let result = PackageJson::parse(data);
        match (&result, &expected) {
            (Ok(result), Ok(expected)) => assert_eq!(result, expected),
            (Err(result), Err(expected)) => assert!(true, "{:?} result did not match expected {:?}", result, expected),
            _ => {
                assert!(false, "{:?} result did not match expected {:?}", result, expected);
            }
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
            (Err(result), Err(expected)) => assert_eq!(result.to_string(), expected.to_string()),
            _ => {
                assert!(false, "{:?} result did not match expected {:?}", result, expected);
            }
        }
    }
}
