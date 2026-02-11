mod cargo_toml;
mod package_json;
mod pyproject_toml;
mod supported_manifests;

pub use cargo_toml::CargoToml;
pub use package_json::PackageJson;
pub use pyproject_toml::PyProjectToml;
pub use supported_manifests::SupportedManifest;

use crate::ManifestStatic;

pub fn manifest_search_order() -> [&'static str; 3] {
    [
        PyProjectToml::manifest_filename(),
        PackageJson::manifest_filename(),
        CargoToml::manifest_filename(),
    ]
}
