use std::path::{Path, PathBuf};

use xdg::BaseDirectories;

use crate::{find_manifest, find_top_of_repo, ConfigError, SemRelConfig};
pub const DEFAULT_CONFIG_FILENAME: &str = ".semrel.toml";

pub fn find_local_config_path(path: impl AsRef<Path>) -> Option<PathBuf> {
    tracing::debug!("Searching for configuration file under: {}", path.as_ref().display());
    let paths = build_config_paths(path).ok().unwrap_or_default();
    let result = paths
        .iter()
        .cloned()
        .inspect(|p| tracing::trace!("Searching for config file: {}", p.display()))
        .find(|p| p.exists());
    match result {
        Some(path) => {
            tracing::debug!("Found configuration: {}", path.display());
            Some(path)
        }
        None => {
            tracing::debug!("No configuration file found.");
            None
        }
    }
}

pub fn find_canonical_config_path() -> Option<PathBuf> {
    let paths = build_canonical_config_paths().ok().unwrap_or_default();
    paths.iter().find(|p| p.exists()).cloned()
}

pub fn load_config(path: impl AsRef<Path>) -> Result<SemRelConfig, ConfigError> {
    // Maybe path _is_ the config?
    if path.as_ref().is_file() {
        tracing::debug!("Loading configuration path: {}", path.as_ref().display());
        let data = std::fs::read_to_string(&path).map_err(|e| ConfigError::InvalidConfig(e.to_string()))?;
        tracing::debug!("Loaded data for configuration path: {}", path.as_ref().display());
        let config: SemRelConfig = match toml::from_str(&data) {
            Ok(config) => config,
            Err(why) => {
                tracing::error!("Could not parse {data}. {why}");
                return Err(ConfigError::InvalidConfig(why.to_string()));
            }
        };
        tracing::debug!("Built config data for configuration path: {}", path.as_ref().display());
        tracing::trace!("Config = {:?}", config);
        Ok(config)
    } else {
        let path = match find_local_config_path(path).or_else(find_canonical_config_path) {
            Some(p) => p,
            None => {
                tracing::debug!("No configuration file found, using default configuration");
                return Ok(SemRelConfig::default());
            }
        };
        tracing::trace!("Loading configuration: {}", path.display());
        let data = match std::fs::read_to_string(&path) {
            Ok(data) => {
                tracing::trace!("Successfully read: {}", path.display());
                data
            }
            Err(why) => {
                tracing::error!("Could not read configuration file: {}.  {why}", path.display());
                return Err(ConfigError::InvalidConfig(why.to_string()));
            }
        };
        let config = match toml::from_str::<SemRelConfig>(&data) {
            Ok(config) => {
                let rules = config.rules().into_iter().collect::<Vec<_>>();
                match rules.is_empty() {
                    true => return Err(ConfigError::EmptyConfig(path.clone())),
                    false => config,
                }
            }
            Err(why) => {
                tracing::error!("Could not parse configuration file: {}.  {why}", path.display());
                return Err(ConfigError::InvalidConfig(why.to_string()));
            }
        };
        Ok(config)
    }
}

fn build_config_paths(path: impl AsRef<Path>) -> Result<Vec<PathBuf>, ConfigError> {
    let manifest_path = find_manifest(&path)?;
    let project_path = manifest_path.parent().unwrap();
    let repo_path = find_top_of_repo(&path)?;

    let mut paths = vec![
        // Next to the manifest file
        project_path.with_file_name(DEFAULT_CONFIG_FILENAME),
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
