use core::fmt;
use std::path::{Path, PathBuf};

use crate::{CargoToml, Manifest, ManifestError, ManifestObjectSafe, ManifestStatic, PackageJson, PyProjectToml, SimpleVersion};

#[derive(Debug, Default)]
pub enum SupportedManifest {
    #[default]
    Unsupported,
    Rust(Box<CargoToml>),
    Javascript(Box<PackageJson>),
    Python(Box<PyProjectToml>),
}

impl SupportedManifest {
    pub fn filename(&self) -> Result<&'static str, ManifestError> {
        tracing::trace!("Getting filename from manifest");
        let filename = match self {
            SupportedManifest::Rust(manifest) => Ok(manifest.filename()),
            SupportedManifest::Javascript(manifest) => Ok(manifest.filename()),
            SupportedManifest::Python(manifest) => Ok(manifest.filename()),
            SupportedManifest::Unsupported => Err(ManifestError::InvalidManifest(self.to_string())),
        };
        tracing::trace!("Filename: {:?}", filename);
        filename
    }

    pub fn version(&self) -> Result<SimpleVersion, ManifestError> {
        tracing::trace!("Getting version from manifest");
        let version = match self {
            SupportedManifest::Rust(manifest) => manifest.version(),
            SupportedManifest::Javascript(manifest) => manifest.version(),
            SupportedManifest::Python(manifest) => manifest.version(),
            SupportedManifest::Unsupported => Err(ManifestError::InvalidManifest(self.to_string())),
        };
        tracing::trace!("Version: {:?}", version);
        version
    }

    pub fn parse(path: impl AsRef<Path>, data: impl AsRef<str>) -> Result<Self, ManifestError> {
        let path = path.as_ref();
        let data = data.as_ref();
        tracing::trace!("Parsing manifest: {:?}", path);
        let package_json = PackageJson::manifest_filename();
        let cargo_toml = CargoToml::manifest_filename();
        let pyproject_toml = PyProjectToml::manifest_filename();
        let parsed = match path.file_name().unwrap().to_str().unwrap() {
            p if p.contains(package_json) => SupportedManifest::Javascript(Box::new(PackageJson::parse(data)?)),
            p if p.contains(cargo_toml) => SupportedManifest::Rust(Box::new(CargoToml::parse(data)?)),
            p if p.contains(pyproject_toml) => SupportedManifest::Python(Box::new(PyProjectToml::parse(data)?)),
            _ => return Err(ManifestError::InvalidManifestPath(path.to_path_buf())),
        };
        tracing::trace!("Parsed manifest version: {:?}", parsed.version()?);
        Ok(parsed)
    }

    pub fn set_version(&mut self, version: impl Into<SimpleVersion>) -> Result<(), ManifestError> {
        tracing::trace!("Setting version in manifest");
        match self {
            SupportedManifest::Rust(manifest) => manifest.set_version(version)?,
            SupportedManifest::Javascript(manifest) => manifest.set_version(version)?,
            SupportedManifest::Python(manifest) => manifest.set_version(version)?,
            SupportedManifest::Unsupported => Err(ManifestError::InvalidManifest(self.to_string()))?,
        }
        Ok(())
    }

    pub fn write(&self, path: impl Into<PathBuf>) -> Result<(), ManifestError> {
        tracing::trace!("Writing manifest");
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
            if value.exists() {
                let data = std::fs::read_to_string(&value).map_err(|_| ManifestError::InvalidManifestPath(value.clone()))?;
                SupportedManifest::parse(value, data)
            } else {
                Err(ManifestError::InvalidManifestPath(value))
            }
        } else {
            Err(ManifestError::InvalidManifestPath(value))
        }
    }
}
