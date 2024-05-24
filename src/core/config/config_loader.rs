use std::path::{PathBuf, Path};

use xdg::BaseDirectories;

use crate::{find_manifest, top_of_repo, ConfigError, BumpRuleConfig};
pub const DEFAULT_CONFIG_FILENAME: &str = ".semrel.toml";


pub fn find_local_config_path(path: impl AsRef<Path>) -> Option<PathBuf> {
    let paths = build_config_paths(path).ok().unwrap_or_default();
    paths.iter().cloned().find(|p| p.exists())
}

pub fn find_canonical_config_path() -> Option<PathBuf> {
    let paths = build_canonical_config_paths().ok().unwrap_or_default();
    paths.iter().cloned().find(|p| p.exists())
}

pub fn load_config(path: impl AsRef<Path>) -> Result<BumpRuleConfig, ConfigError> {
    // Maybe path _is_ the config?
    if path.as_ref().is_file() {
        let data = std::fs::read_to_string(&path).map_err(|e| ConfigError::InvalidConfig(e.to_string()))?;
        let config: BumpRuleConfig = toml::from_str(&data).map_err(|e| ConfigError::InvalidConfig(e.to_string()))?;
        return Ok(config)
    } else {
        let path = match find_local_config_path(path).or_else(find_canonical_config_path) {
            Some(p) => p,
            None => {
                tracing::debug!("No configuration file found, using default configuration");
                return Ok(BumpRuleConfig::default())
            }
        };
        let data = std::fs::read_to_string(&path).map_err(|e| ConfigError::InvalidConfig(e.to_string()))?;
        let config: BumpRuleConfig = toml::from_str(&data).map_err(|e| ConfigError::InvalidConfig(e.to_string()))?;
        Ok(config)
    }
}

fn build_config_paths(path: impl AsRef<Path>) -> Result<Vec<PathBuf>, ConfigError> {
    let manifest_path = find_manifest(&path)?;
    let repo_path = top_of_repo(&path)?;

    let mut paths = vec![
        // Next to the manifest file
        manifest_path.with_file_name(DEFAULT_CONFIG_FILENAME),
        // At the root of the project
        repo_path.join(DEFAULT_CONFIG_FILENAME),
    ];
    paths.extend(build_canonical_config_paths()?);
    Ok(paths)
}

fn build_canonical_config_paths() -> Result<Vec<PathBuf>, ConfigError> {
    let xdg_dirs = BaseDirectories::with_prefix("semrel").map_err(|e| ConfigError::InvalidConfig(e.to_string()))?;

    let paths = [
        // In an XDG compliant configuration directory
        xdg_dirs.find_config_file("config.toml").unwrap_or_else(|| {
            // Under $HOME/.config/semrel/config.toml (if $XDG_CONFIG_HOME is not set)
            dirs::home_dir().unwrap().join(".config/semrel/config.toml")
        }),
        // In the system configuration directory
        PathBuf::from("/etc/semrel/config.toml"),
    ];

    Ok(paths.to_vec())
}