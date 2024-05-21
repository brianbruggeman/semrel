use crate::{CargoToml, PackageJson, PyProjectToml};

#[derive(Debug, Default)]
pub enum SupportedManifest {
    #[default]
    Unsupported,
    Rust(CargoToml),
    Javascript(PackageJson),
    Python(PyProjectToml),
}