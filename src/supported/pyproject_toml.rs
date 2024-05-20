use crate::{
    core::{Manifest, ManifestError, ManifestObjectSafe, SimpleVersion},
    ManifestStatic,
};

#[derive(Debug, serde::Deserialize, PartialEq, Eq, Clone, Copy)]
pub struct PoetryPackage {
    pub version: SimpleVersion,
}

impl Default for PoetryPackage {
    fn default() -> Self {
        Self { version: SimpleVersion::new(0, 1, 0) }
    }
}

#[derive(Debug, Default, serde::Deserialize, PartialEq, Eq, Clone, Copy)]
pub struct PyProjectTool {
    pub poetry: PoetryPackage,
}

#[derive(Debug, serde::Deserialize, PartialEq, Eq, Clone, Copy)]
pub struct Pep621Package {
    pub version: SimpleVersion,
}

impl Default for Pep621Package {
    fn default() -> Self {
        Self { version: SimpleVersion::new(0, 1, 0) }
    }
}

#[derive(Debug, Default, serde::Deserialize, PartialEq, Eq, Clone, Copy)]
pub struct PyProjectToml {
    pub project: Option<Pep621Package>,
    pub tool: Option<PyProjectTool>,
}

impl PyProjectToml {
    pub fn new(version: impl Into<SimpleVersion>) -> Self {
        Self {
            project: Some(Pep621Package { version: version.into() }),
            tool: None,
        }
    }
}

impl ManifestStatic for PyProjectToml {
    fn manifest_filename() -> &'static str {
        "pyproject.toml"
    }
}

impl ManifestObjectSafe for PyProjectToml {
    fn version(&self) -> Result<SimpleVersion, ManifestError> {
        if let Some(pep621) = self.project {
            return Ok(pep621.version);
        } else if let Some(tool) = self.tool {
            return Ok(tool.poetry.version);
        }
        Err(ManifestError::InvalidManifest("Missing version".to_string()))
    }
}

impl Manifest for PyProjectToml {
    fn parse(data: impl AsRef<str>) -> Result<Self, ManifestError> {
        // Validate this can be read and is a valid json
        let package: Self = toml::from_str(data.as_ref()).map_err(|why| ManifestError::InvalidManifest(why.to_string()))?;
        Ok(package)
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
    #[case::pep621_validate_valid_json("[project]\nversion = \"1.0.0\"", Ok(PyProjectToml { project: Some(Pep621Package { version: SimpleVersion::new(1, 0, 0) }), tool: None }))]
    #[case::pep621_validate_invalid_json("[project]\nversion = \"invalid-version\"", Err(ManifestError::InvalidManifest("Invalid manifest: Invalid version part: invalid digit found in string at line 1 column 37".to_string())))]
    #[case::poetry_validate_valid_json("[tool.poetry]\nversion = \"1.0.0\"", Ok(PyProjectToml { project: None, tool: Some(PyProjectTool { poetry: PoetryPackage { version: SimpleVersion::new(1, 0, 0) } }) }))]
    #[case::poetry_validate_invalid_json("[tool.poetry]\nversion = \"invalid-version\"", Err(ManifestError::InvalidManifest("Invalid manifest: Invalid version part: invalid digit found in string at line 1 column 37".to_string())))]
    fn test_parse(#[case] data: &str, #[case] expected: Result<PyProjectToml, ManifestError>) {
        let result = PyProjectToml::parse(data);
        match (&result, &expected) {
            (Ok(result), Ok(expected)) => assert_eq!(result, expected),
            (Err(_result), Err(_expected)) => {}
            _ => panic!("{:?} result did not match expected {:?}", result, expected),
        }
    }

    #[rstest]
    #[case::pep621_parse_valid_version("[project]\nversion = \"1.0.0\"", Ok(SimpleVersion::new(1, 0, 0)))]
    #[case::pep621_parse_invalid_version("[project]\nversion = \"invalid-version\"", Err(ManifestError::InvalidManifest("TOML parse error at line 2, column 11\n  |\n2 | version = \"invalid-version\"\n  |           ^^^^^^^^^^^^^^^^^\nInvalid version part: invalid digit found in string\n".to_string())))]
    #[case::pep621_parse_missing_version("[project]\nname = \"pep621-package\"", Err(ManifestError::InvalidManifest("TOML parse error at line 1, column 1\n  |\n1 | [project]\n  | ^^^^^^^^^\nmissing field `version`\n".to_string())))]
    #[case::poetry_parse_valid_version("[tool.poetry]\nversion = \"1.0.0\"", Ok(SimpleVersion::new(1, 0, 0)))]
    #[case::poetry_parse_invalid_version("[tool.poetry]\nversion = \"invalid-version\"", Err(ManifestError::InvalidManifest("TOML parse error at line 2, column 11\n  |\n2 | version = \"invalid-version\"\n  |           ^^^^^^^^^^^^^^^^^\nInvalid version part: invalid digit found in string\n".to_string())))]
    #[case::poetry_parse_missing_version("[tool.poetry]\nname = \"poetry-package\"", Err(ManifestError::InvalidManifest("TOML parse error at line 1, column 1\n  |\n1 | [tool.poetry]\n  | ^^^^^^^^^^^^^\nmissing field `version`\n".to_string())))]
    fn test_parse_version(#[case] data: &str, #[case] expected: Result<SimpleVersion, ManifestError>) {
        let result = PyProjectToml::parse_version(data);
        match (&result, &expected) {
            (Ok(result), Ok(expected)) => assert_eq!(result, expected),
            (Err(result), Err(expected)) => assert_eq!(result.to_string(), expected.to_string()),
            _ => panic!("{:?} result did not match expected {:?}", result, expected),
        }
    }
}
