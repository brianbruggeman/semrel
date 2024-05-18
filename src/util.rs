use crate::{CargoToml, Manifest, ManifestError, ManifestObjectSafe, ManifestStatic, PackageJson, PyProjectToml};
use std::path::{Path, PathBuf};

pub fn find_manifest(path: impl AsRef<Path>) -> Result<PathBuf, ManifestError> {
    [
        path.as_ref().to_path_buf(),
        path.as_ref().join(PyProjectToml::manifest_filename()),
        path.as_ref().join(PackageJson::manifest_filename()),
        path.as_ref().join(CargoToml::manifest_filename()),
    ]
    .into_iter()
    .inspect(|path| {tracing::debug!("Checking for: {}", path.display());})
    .find(|path| path.exists() && path.is_file())
    .inspect(|path| {tracing::debug!("Found: {}", path.display());})
    .ok_or_else(|| ManifestError::InvalidManifestPath(path.as_ref().to_path_buf()))
}

pub fn parse_manifest(path: impl AsRef<Path>) -> Result<Box<dyn ManifestObjectSafe>, ManifestError> {
    tracing::debug!("Searching: {}", path.as_ref().display());
    let manifest_path = find_manifest(path)?;
    tracing::debug!("Found: {}", manifest_path.display());
    let data = std::fs::read_to_string(&manifest_path).map_err(|why| ManifestError::InvalidManifest(why.to_string()))?;
    let manifest_filename = match manifest_path.file_name() {
        Some(filename) => filename.to_string_lossy(),
        None => return Err(ManifestError::InvalidManifestPath(manifest_path)),
    };
    tracing::debug!("Found manifest file: {}", manifest_filename);

    match manifest_filename {
        f if f == CargoToml::manifest_filename() => Ok(Box::new(CargoToml::parse(data)?)),
        f if f == PackageJson::manifest_filename() => Ok(Box::new(PackageJson::parse(data)?)),
        f if f == PyProjectToml::manifest_filename() => Ok(Box::new(PyProjectToml::parse(data)?)),
        _ => Err(ManifestError::InvalidManifestPath(manifest_path)),
    }
}
