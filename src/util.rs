use crate::{CargoToml, Manifest, ManifestError, ManifestObjectSafe, ManifestStatic, PackageJson, PyProjectToml};
use std::path::{Path, PathBuf};

pub fn find_manifest(path: impl AsRef<Path>) -> Result<PathBuf, ManifestError> {
    [
        path.as_ref().join(CargoToml::manifest_filename()),
        path.as_ref().join(PackageJson::manifest_filename()),
        path.as_ref().join(PyProjectToml::manifest_filename()),
    ]
    .iter()
    .find(|path| path.exists())
    .map(|path| path.to_path_buf())
    .ok_or_else(|| ManifestError::InvalidManifestPath(path.as_ref().to_path_buf()))
}

pub fn parse_manifest(path: impl AsRef<Path>) -> Result<Box<dyn ManifestObjectSafe>, ManifestError> {
    let manifest_path = find_manifest(path)?;
    let data = std::fs::read_to_string(&manifest_path).map_err(|why| ManifestError::InvalidManifest(why.to_string()))?;

    if manifest_path.file_name().unwrap() == CargoToml::manifest_filename() {
        Ok(Box::new(CargoToml::parse(data)?))
    } else if manifest_path.file_name().unwrap() == PackageJson::manifest_filename() {
        Ok(Box::new(PackageJson::parse(data)?))
    } else if manifest_path.file_name().unwrap() == PyProjectToml::manifest_filename() {
        Ok(Box::new(PyProjectToml::parse(data)?))
    } else {
        Err(ManifestError::InvalidManifestPath(manifest_path))
    }
}
