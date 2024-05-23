use core::fmt;
use std::path::{Path, PathBuf};

use crate::{CargoToml, Manifest, ManifestError, ManifestObjectSafe, ManifestStatic, PackageJson, PyProjectToml, SimpleVersion};

#[derive(Debug, Default)]
pub enum SupportedManifest {
    #[default]
    Unsupported,
    Rust(CargoToml),
    Javascript(PackageJson),
    Python(PyProjectToml),
}

impl SupportedManifest {
    pub fn filename(&self) -> Result<&'static str, ManifestError> {
        tracing::debug!("Getting filename from manifest");
        let filename = match self {
            SupportedManifest::Rust(manifest) => Ok(manifest.filename()),
            SupportedManifest::Javascript(manifest) => Ok(manifest.filename()),
            SupportedManifest::Python(manifest) => Ok(manifest.filename()),
            SupportedManifest::Unsupported => Err(ManifestError::InvalidManifest(self.to_string())),
        };
        tracing::debug!("Filename: {:?}", filename);
        filename
    }

    pub fn version(&self) -> Result<SimpleVersion, ManifestError> {
        tracing::debug!("Getting version from manifest");
        let version = match self {
            SupportedManifest::Rust(manifest) => manifest.version(),
            SupportedManifest::Javascript(manifest) => manifest.version(),
            SupportedManifest::Python(manifest) => manifest.version(),
            SupportedManifest::Unsupported => Err(ManifestError::InvalidManifest(self.to_string())),
        };
        tracing::debug!("Version: {:?}", version);
        version
    }

    pub fn parse(path: impl AsRef<Path>, data: impl AsRef<str>) -> Result<Self, ManifestError> {
        let path = path.as_ref();
        let data = data.as_ref();
        tracing::debug!("Parsing manifest: {:?}", path);
        let package_json = PackageJson::manifest_filename();
        let cargo_toml = CargoToml::manifest_filename();
        let pyproject_toml = PyProjectToml::manifest_filename();
        let parsed = match path.file_name().unwrap().to_str().unwrap() {
            p if p.contains(package_json) => SupportedManifest::Javascript(PackageJson::parse(data)?),
            p if p.contains(cargo_toml) => SupportedManifest::Rust(CargoToml::parse(data)?),
            p if p.contains(pyproject_toml) => SupportedManifest::Python(PyProjectToml::parse(data)?),
            _ => return Err(ManifestError::InvalidManifestPath(path.to_path_buf())),
        };
        tracing::debug!("Parsed manifest version: {:?}", parsed.version()?);
        Ok(parsed)
    }

    pub fn set_version(&mut self, version: impl Into<SimpleVersion>) -> Result<(), ManifestError> {
        tracing::debug!("Setting version in manifest");
        match self {
            SupportedManifest::Rust(manifest) => manifest.set_version(version)?,
            SupportedManifest::Javascript(manifest) => manifest.set_version(version)?,
            SupportedManifest::Python(manifest) => manifest.set_version(version)?,
            SupportedManifest::Unsupported => Err(ManifestError::InvalidManifest(self.to_string()))?,
        }
        Ok(())
    }

    pub fn write(&self, path: impl Into<PathBuf>) -> Result<(), ManifestError> {
        tracing::debug!("Writing manifest");
        let path = path.into();
        match self {
            SupportedManifest::Rust(manifest) => manifest.write(path)?,
            SupportedManifest::Javascript(manifest) => manifest.write(path)?,
            SupportedManifest::Python(manifest) => manifest.write(path)?,
            SupportedManifest::Unsupported => Err(ManifestError::InvalidManifest(self.to_string()))?,
        }
        Ok(())
    }
}

impl fmt::Display for SupportedManifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SupportedManifest::Rust(_) => write!(f, "Rust"),
            SupportedManifest::Javascript(_) => write!(f, "Javascript"),
            SupportedManifest::Python(_) => write!(f, "Python"),
            SupportedManifest::Unsupported => write!(f, "Unsupported"),
        }
    }
}

impl TryFrom<PathBuf> for SupportedManifest {
    type Error = ManifestError;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let valid_manifests = [CargoToml::manifest_filename(), PackageJson::manifest_filename(), PyProjectToml::manifest_filename()];
        if value.is_dir() {
            for manifest in valid_manifests.iter() {
                let manifest_path = value.join(manifest);
                if manifest_path.exists() {
                    let data = std::fs::read_to_string(&manifest_path).map_err(|_| ManifestError::InvalidManifestPath(manifest_path.clone()))?;
                    return SupportedManifest::parse(manifest_path, data);
                }
            }
            Err(ManifestError::InvalidManifestPath(value))
        } else if value.is_file() {
            SupportedManifest::parse(value, "")
        } else {
            Err(ManifestError::InvalidManifestPath(value))
        }
    }
}
