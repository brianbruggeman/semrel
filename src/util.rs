use crate::{CargoToml, ManifestError, ManifestStatic, PackageJson, PyProjectToml, SupportedManifest};
use std::path::{Path, PathBuf};

pub fn find_manifest(path: impl AsRef<Path>) -> Result<PathBuf, ManifestError> {
    [
        path.as_ref().to_path_buf(),
        path.as_ref().join(PyProjectToml::manifest_filename()),
        path.as_ref().join(PackageJson::manifest_filename()),
        path.as_ref().join(CargoToml::manifest_filename()),
    ]
    .into_iter()
    .inspect(|path| {
        tracing::debug!("Checking for manifest under: {}", path.display());
    })
    .find(|path| path.exists() && path.is_file())
    .inspect(|path| {
        tracing::debug!("Found manifest under: {}", path.display());
    })
    .ok_or_else(|| ManifestError::InvalidManifestPath(path.as_ref().to_path_buf()))
}

pub fn parse_manifest(path: impl AsRef<Path>) -> Result<SupportedManifest, ManifestError> {
    let mut path = path.as_ref().to_owned();
    if !path.is_file() {
        path = find_manifest(path)?;
    }
    let data = std::fs::read_to_string(&path).map_err(|why| ManifestError::InvalidManifest(why.to_string()))?;
    tracing::debug!("Reading manifest file: {}", path.display());
    SupportedManifest::parse(path, data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};

    #[fixture]
    fn temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[rstest]
    #[case::cargo_toml("[package]\nversion = \"0.1.0\"", "Cargo.toml")]
    #[case::package_json("{\n  \"version\": \"0.1.0\"\n}", "package.json")]
    #[case::poetry("[tool.poetry]\nversion = \"0.1.0\"", "pyproject.toml")]
    #[case::pep621("[project]\nversion = \"0.1.0\"", "pyproject.toml")]
    fn test_find_manifest(temp_dir: tempfile::TempDir, #[case] manifest: &str, #[case] filename: &str) {
        let manifest_path = temp_dir.path().join(filename);
        std::fs::write(&manifest_path, manifest).unwrap();
        let path = temp_dir.path();
        let found = find_manifest(path).unwrap();
        assert_eq!(found, manifest_path);
    }

    #[rstest]
    #[case::cargo_toml("[package]\nname = \"test\"\nversion = \"0.1.0\"", "Cargo.toml", "0.1.0")]
    #[case::package_json("{\n  \"name\": \"test\",\n  \"version\": \"0.1.0\"\n}", "package.json", "0.1.0")]
    #[case::poetry("[tool.poetry]\nname = \"test\"\nversion = \"0.1.0\"", "pyproject.toml", "0.1.0")]
    #[case::pep621("[project]\nname = \"test\"\nversion = \"0.1.0\"", "pyproject.toml", "0.1.0")]
    fn test_parse_manifest(temp_dir: tempfile::TempDir, #[case] manifest: &str, #[case] filename: &str, #[case] version: &str) {
        let manifest_path = temp_dir.path().join(filename);
        std::fs::write(&manifest_path, manifest).unwrap();
        let path = temp_dir.path();
        let manifest = parse_manifest(path).unwrap();
        assert_eq!(manifest.version().unwrap(), version);
    }
}
