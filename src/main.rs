use std::path::Path;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use semrel::*;

#[derive(Debug, clap::Parser)]
pub struct Opts {
    /// Path to the project root
    #[clap(long, default_value = ".", global = true)]
    path: String,
    /// Custom rules for commit types (can be comma separated)
    #[clap(long, num_args(0..), global = true)]
    rule: Vec<String>,
    /// Short circuit for bumping the version
    #[clap(short, long, global = true)]
    bump: Option<BumpRule>,
    /// Specify the configuration path
    #[clap(long, global = true)]
    config_path: Option<String>,

    #[clap(subcommand)]
    pub cmd: Command,
}


#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Manage versioning
    Version {
        #[clap(subcommand)]
        cmd: VersionOpts,
    },
    /// Show commit log used to calculate the version
    Log,
    /// Show release notes
    Notes,
    /// Show the path to the manifest file
    Manifest,
    /// Show the current rules in order of precedence
    Rules,
    /// Config subcommand
    Config {
        #[clap(subcommand)]
        cmd: ConfigOpts,
    },
}


#[derive(Debug, clap::Subcommand)]
pub enum VersionOpts {
    /// Show only the current version
    Current,
    /// Show the next version
    Next,
}

#[derive(Debug, clap::Subcommand)]
pub enum ConfigOpts {
    /// Show the current configuration
    Show,
    /// Edit the current configuration
    Edit,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();
    let opts = Opts::parse();

    let path = &opts.path;
    let repo = get_repo(path).map_err(|_| RepositoryError::InvalidRepositoryPath(path.into()))?;
    let config_rules = load_config(path).unwrap_or_default();
    let rules = parse_rules(&opts.rule)?.chain(config_rules).chain(build_default_rules()).collect::<Vec<_>>();
    let changelog = get_changelog(&repo, &rules)?;
    let current_version = changelog.current_version;
    let bump = opts.bump.unwrap_or_default();
    let new_version = match bump {
        BumpRule::Notset => changelog.next_version(&rules),
        _ => changelog.current_version.bump(bump),
    };

    match opts.cmd {
        Command::Version{cmd } => {
            handle_version_command(cmd, &new_version, &current_version)
        }
        Command::Log => {
            for commit_info in changelog.changes {
                println!("{} {}", commit_info.id, commit_info.message());
            }
            Ok(())
        }
        Command::Notes => {
            println!("{}", changelog.release_notes(&rules));
            Ok(())
        }
        Command::Manifest => {
            let manifest_path = find_manifest(path).map_err(|_| RepositoryError::InvalidManifestPath(path.into()))?;
            println!("{}", manifest_path.display());
            Ok(())
        }
        Command::Rules => {
            for (ct, br) in rules {
                println!("{}: {:?}", ct, br);
            }
            Ok(())
        }
        Command::Config{ cmd } => {
            handle_config_command(cmd, path)
        }
    }
}

fn handle_config_command(cmd: ConfigOpts, path: impl AsRef<Path>) -> anyhow::Result<()> {
    match cmd {
        ConfigOpts::Show => {
            let config = load_config(path.as_ref())?;
            println!("{:?}", config);
            Ok(())
        }
        ConfigOpts::Edit => {
            let config_path = find_local_config_path(path.as_ref()).or_else(find_canonical_config_path).ok_or_else(|| ConfigError::ConfigNotFound(path.as_ref().to_owned()))?;
            // Interactively run the default editor if it is set
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
            let status = std::process::Command::new(editor).arg(config_path).status()?;
            if !status.success() {
                return Err(anyhow::anyhow!("Failed to open editor"));
            }
            Ok(())
        }
    }
}

fn handle_version_command(cmd: VersionOpts, new_version: &SimpleVersion, current_version: &SimpleVersion) -> anyhow::Result<()> {
    match cmd {
        VersionOpts::Current => {
            println!("{current_version}");
            Ok(())
        }
        VersionOpts::Next => {
            println!("{new_version}");
            Ok(())
        }
    }
}