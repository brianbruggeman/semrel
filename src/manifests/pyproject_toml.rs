use std::path::PathBuf;
use std::str::FromStr;

use crate::{
    ManifestStatic,
    core::{Manifest, ManifestError, SimpleVersion},
};

#[derive(Debug)]
pub struct PyProjectToml {
    manifest: toml::Value,
    raw: String,
}

impl PyProjectToml {
    pub fn new(version: impl Into<SimpleVersion>) -> Self {
        let mut pep621_manifest = Self::default();
        pep621_manifest.set_pep621_version(version);
        pep621_manifest
    }

    fn set_pep621_version(&mut self, version: impl Into<SimpleVersion>) -> bool {
        let version_string = version.into().to_string();
        if let Some(project) = self.manifest.get_mut("project") {
            if let Some(project_table) = project.as_table_mut() {
                project_table.insert("version".to_string(), toml::Value::String(version_string));
                return true;
            }
        }
        false
    }

    fn set_poetry_version(&mut self, version: impl Into<SimpleVersion>) -> bool {
        let version_string = version.into().to_string();
        if let Some(tool) = self.manifest.get_mut("tool") {
            if let Some(tool_table) = tool.as_table_mut() {
                if let Some(poetry) = tool_table.get_mut("poetry") {
                    if let Some(poetry_table) = poetry.as_table_mut() {
                        poetry_table.insert("version".to_string(), toml::Value::String(version_string));
                        return true;
                    }
                }
            }
        }
        false
    }

    fn get_pep621_version(&self) -> Result<Option<SimpleVersion>, ManifestError> {
        if let Some(project) = &self.manifest.get("project") {
            if let Some(version) = project.get("version") {
                if let Some(version_str) = version.as_str() {
                    return SimpleVersion::from_str(version_str)
                        .map(Some)
                        .map_err(ManifestError::InvalidManifestVersion);
                }
            }
        }
        Ok(None)
    }

    fn get_poetry_version(&self) -> Result<Option<SimpleVersion>, ManifestError> {
        if let Some(tool) = &self.manifest.get("tool") {
            if let Some(poetry) = tool.get("poetry") {
                if let Some(version) = poetry.get("version") {
                    if let Some(version_str) = version.as_str() {
                        return SimpleVersion::from_str(version_str)
                            .map(Some)
                            .map_err(ManifestError::InvalidManifestVersion);
                    }
                }
            }
        }
        Ok(None)
    }
}

impl Default for PyProjectToml {
    fn default() -> Self {
        let pep621_data = r#"
            [project]
            name = "pep621-package"
            version = "0.1.0"
        "#;
        let manifest = toml::from_str(pep621_data).expect("hardcoded TOML must be valid");
        Self {
            manifest,
            raw: pep621_data.to_string(),
        }
    }
}

impl ManifestStatic for PyProjectToml {
    fn manifest_filename() -> &'static str {
        "pyproject.toml"
    }
}

impl Manifest for PyProjectToml {
    fn version(&self) -> Result<SimpleVersion, ManifestError> {
        if let Some(version) = self.get_pep621_version()? {
            return Ok(version);
        }
        if let Some(version) = self.get_poetry_version()? {
            return Ok(version);
        }
        Err(ManifestError::InvalidManifest("No version found".to_string()))
    }

    fn set_version(&mut self, version: impl Into<SimpleVersion>) -> Result<(), ManifestError> {
        let version = version.into();
        if self.set_pep621_version(version) || self.set_poetry_version(version) {
            return Ok(());
        }
        Err(ManifestError::InvalidManifest("No version found".to_string()))
    }

    fn write(&self, path: impl Into<PathBuf>) -> Result<(), ManifestError> {
        let version = self.version()?.to_string();
        let mut doc: toml_edit::DocumentMut = self
            .raw
            .parse()
            .map_err(|why: toml_edit::TomlError| ManifestError::InvalidManifest(why.to_string()))?;
        if doc.get("project").is_some() {
            doc["project"]["version"] = toml_edit::value(&version);
        }
        if doc.get("tool").and_then(|t| t.get("poetry")).is_some() {
            doc["tool"]["poetry"]["version"] = toml_edit::value(&version);
        }
        std::fs::write(path.into(), doc.to_string()).map_err(|why| ManifestError::WriteError(why.to_string()))
    }

    fn parse(data: impl AsRef<str>) -> Result<Self, ManifestError> {
        tracing::debug!("Parsing pyproject.toml");
        let raw = data.as_ref().to_string();
        let manifest = toml::from_str(&raw).map_err(|why| ManifestError::InvalidManifest(why.to_string()))?;
        Ok(Self { manifest, raw })
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
    use tempfile::{TempDir, tempdir};

    #[fixture]
    fn temp_pyproject_toml() -> (TempDir, PathBuf, PathBuf) {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("pyproject.toml");

        let parent = file_path.parent().unwrap().to_path_buf();
        (temp_dir, parent, file_path)
    }

    #[test]
    fn test_pep621_find_valid_pyproject_toml() {
        let (_temp_dir, parent, pyproject_toml_path) = temp_pyproject_toml();
        let mut file = File::create(&pyproject_toml_path).unwrap();
        let data = "[project]\nversion = \"1.0.0\"";
        write!(file, "{data}").unwrap();

        // Test finding the pyproject.toml file
        let result = PyProjectToml::find(&pyproject_toml_path);
        assert!(result.is_ok(), "Expected to find pyproject.toml, but got {:?}", result);
        assert_eq!(result.unwrap(), pyproject_toml_path);

        // Test finding the pyproject.toml file from the parent directory
        let result = PyProjectToml::find(parent);
        assert!(result.is_ok(), "Expected to find pyproject.toml, but got {:?}", result);
        assert_eq!(result.unwrap(), pyproject_toml_path);
    }

    #[test]
    fn test_pep621_load_valid_file() {
        let (_temp_dir, _parent, pyproject_toml_path) = temp_pyproject_toml();
        let mut file = File::create(&pyproject_toml_path).unwrap();
        let data = "[project]\nversion = \"1.0.0\"";
        write!(file, "{data}").unwrap();

        let result = PyProjectToml::load(&pyproject_toml_path);
        let data = "[project]\nversion = \"1.0.0\"";
        assert!(result.is_ok(), "Expected to load pyproject.toml, but got {:?}", result);
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_find_valid_pyproject_toml() {
        let (_temp_dir, parent, pyproject_toml_path) = temp_pyproject_toml();
        let mut file = File::create(&pyproject_toml_path).unwrap();
        let data = "[tool.poetry]\nversion = \"1.0.0\"";
        write!(file, "{data}").unwrap();
        // Test finding the pyproject.toml file
        let result = PyProjectToml::find(&pyproject_toml_path);
        assert!(result.is_ok(), "Expected to find pyproject.toml, but got {:?}", result);
        assert_eq!(result.unwrap(), pyproject_toml_path);
        // Test finding the pyproject.toml file from the parent directory
        let result = PyProjectToml::find(parent);
        assert!(result.is_ok(), "Expected to find pyproject.toml, but got {:?}", result);
        assert_eq!(result.unwrap(), pyproject_toml_path);
    }

    #[rstest]
    #[case::poetry_validate_valid_json("[tool.poetry]\nversion = \"1.0.0\"")]
    #[case::poetry_validate_invalid_json("[tool.poetry]\nversion = \"invalid-version\"")]
    #[case::pep621_validate_valid_json("[project]\nversion = \"1.0.0\"")]
    #[case::pep621_validate_invalid_json("[project]\nversion = \"invalid-version\"")]
    fn test_load_valid_file(#[case] data: impl AsRef<str>) {
        let (_temp_dir, _parent, pyproject_toml_path) = temp_pyproject_toml();
        let mut file = File::create(&pyproject_toml_path).unwrap();
        write!(file, "{}", data.as_ref()).unwrap();
        let result = PyProjectToml::load(&pyproject_toml_path);
        assert!(result.is_ok(), "Expected to load pyproject.toml, but got {:?}", result);
        assert_eq!(result.unwrap(), data.as_ref());
    }

    #[test]
    fn test_find_missing_pyproject_toml() {
        let (_temp_dir, parent, _) = temp_pyproject_toml();
        let result = PyProjectToml::find(parent);
        assert!(result.is_err(), "Expected to not find pyproject.toml, but got {:?}", result);
    }

    #[rstest]
    #[case::pep621_validate_valid_json("[project]\nversion = \"1.0.0\"")]
    #[case::pep621_validate_invalid_json("[project]\nversion = \"invalid-version\"")]
    #[case::poetry_validate_valid_json("[tool.poetry]\nversion = \"1.0.0\"")]
    #[case::poetry_validate_invalid_json("[tool.poetry]\nversion = \"invalid-version\"")]
    fn test_parse(#[case] data: &str) {
        let expected = toml::from_str::<toml::Value>(data).map_err(|why| ManifestError::InvalidManifest(why.to_string()));
        let result = PyProjectToml::parse(data);
        match (&result, &expected) {
            (Ok(result), Ok(expected)) => assert_eq!(result.manifest, *expected),
            (Err(result_err), Err(expected_err)) => assert_eq!(result_err.to_string(), expected_err.to_string(), "Error mismatch"),
            _ => panic!("{:?} result did not match expected {:?}", result, expected),
        }
    }

    #[rstest]
    #[case::pep621_parse_valid_version("[project]\nversion = \"1.0.0\"", Ok(SimpleVersion::new(1, 0, 0)))]
    #[case::pep621_parse_invalid_version("[project]\nversion = \"invalid-version\"", Err(ManifestError::InvalidManifestVersion(crate::VersionError::InvalidVersionPart("invalid-version".parse::<u16>().unwrap_err()))))]
    #[case::pep621_parse_missing_version("[project]\nname = \"pep621-package\"", Err(ManifestError::InvalidManifest("No version found".to_string())))]
    #[case::poetry_parse_valid_version("[tool.poetry]\nversion = \"1.0.0\"", Ok(SimpleVersion::new(1, 0, 0)))]
    #[case::poetry_parse_invalid_version("[tool.poetry]\nversion = \"invalid-version\"", Err(ManifestError::InvalidManifestVersion(crate::VersionError::InvalidVersionPart("invalid-version".parse::<u16>().unwrap_err()))))]
    #[case::poetry_parse_missing_version("[tool.poetry]\nname = \"poetry-package\"", Err(ManifestError::InvalidManifest("No version found".to_string())))]
    fn test_parse_version(#[case] data: &str, #[case] expected: Result<SimpleVersion, ManifestError>) {
        let result = PyProjectToml::parse_version(data);
        match (&result, &expected) {
            (Ok(result), Ok(expected)) => assert_eq!(result, expected),
            (Err(result_err), Err(expected_err)) => assert_eq!(result_err.to_string(), expected_err.to_string(), "Error mismatch"),
            _ => panic!("{:?} result did not match expected {:?}", result, expected),
        }
    }
}
