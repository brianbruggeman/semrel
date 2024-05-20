use std::path::{Path, PathBuf};

use super::{ManifestError, SimpleVersion};
use crate::top_of_repo;

pub trait ManifestStatic {
    fn manifest_filename() -> &'static str;
}

pub trait ManifestObjectSafe {
    fn version(&self) -> Result<SimpleVersion, ManifestError>;
}

pub trait Manifest: ManifestStatic + ManifestObjectSafe {
    #[allow(unused_variables)]
    fn parse(data: impl AsRef<str>) -> Result<Self, ManifestError>
    where
        Self: Sized;

    /// This will attempt to determine the manifest path by:
    ///   - checking if the path is the manifest file
    ///   - checking if the path is a folder containing the manifest file
    ///   - checking if the path is within a repository containing the manifest file
    fn find(path: impl AsRef<Path>) -> Result<PathBuf, ManifestError>
    where
        Self: Sized,
    {
        let path = path.as_ref();
        let manifest_filename = Self::manifest_filename();
        if path.is_file() && path.file_name().and_then(|f| f.to_str()) == Some(manifest_filename) {
            Ok(path.to_path_buf())
        } else if path.is_dir() {
            let manifest_path = path.join(manifest_filename);
            if manifest_path.exists() {
                Ok(manifest_path)
            } else {
                let root_path = Self::repo_root(path)?;
                let manifest_path = root_path.join(manifest_filename);
                if manifest_path.exists() {
                    Ok(manifest_path)
                } else {
                    Err(ManifestError::InvalidManifestPath(path.to_path_buf()))
                }
            }
        } else {
            Err(ManifestError::InvalidManifestPath(path.to_path_buf()))
        }
    }

    fn repo_root(path: impl AsRef<Path>) -> Result<PathBuf, ManifestError> {
        top_of_repo(path).map_err(ManifestError::RepositoryError)
    }

    fn load(path: impl AsRef<Path>) -> Result<String, ManifestError>
    where
        Self: Sized,
    {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path.as_ref()).map_err(|why| ManifestError::InvalidManifest(why.to_string()))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|why| ManifestError::InvalidManifest(why.to_string()))?;
        Ok(contents)
    }

    fn parse_version(data: impl AsRef<str>) -> Result<SimpleVersion, ManifestError>
    where
        Self: Sized,
    {
        let package = Self::parse(data)?;
        package.version()
    }
}
