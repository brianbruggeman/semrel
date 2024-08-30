use std::path::PathBuf;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use semrel::*;

#[derive(Debug, clap::Parser)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
pub struct Opts {
    /// Path to the project root
    #[clap(long, default_value = ".", global = true, env = "PROJECT_PATH")]
    path: String,
    /// Custom rules for commit types (can be comma separated)
    #[clap(long, num_args(0..), global = true, env="SEMREL_RULES")]
    rule: Vec<String>,
    /// Short circuit for bumping the version
    #[clap(short, long, global = true, env = "SEMREL_BUMP")]
    bump: Option<BumpRule>,
    /// Specify the configuration path
    #[clap(long, global = true, env = "SEMREL_CONFIG_PATH")]
    config_path: Option<PathBuf>,

    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Update the manifest
    Update,
    /// Show information
    Show {
        #[clap(subcommand)]
        cmd: ShowOpts,
    },
    /// Config subcommand
    Config {
        #[clap(subcommand)]
        cmd: ConfigOpts,
    },
}

#[derive(Debug, clap::Subcommand)]
pub enum ShowOpts {
    /// Show only the current version
    Current,
    /// Show the next version
    Next,
    /// Show the changelog
    Log,
    /// Show the release notes
    Notes,
    /// Show the manifest path
    Manifest,
    /// Show the rules
    Rules,
    /// Show the configuration
    Config,
}

#[derive(Debug, clap::Subcommand)]
pub enum ConfigOpts {
    /// Edit the current configuration
    Edit,
}

struct CliData {
    manifest_path: PathBuf,
    rules: Vec<(CommitType, BumpRule)>,
    config_path: Option<PathBuf>,
    changelog: ChangeLog,
    new_version: SimpleVersion,
    current_version: SimpleVersion,
}

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();
    let opts = Opts::parse();

    let path = &opts.path;
    let repo = get_repo(path).map_err(|_| RepositoryError::InvalidRepositoryPath(path.into()))?;
    let config_path = match opts.config_path.clone() {
        Some(config_path) => {
            tracing::info!("Configuration present in opts: {}", config_path.display());
            Some(config_path)
        }
        None => match find_local_config_path(path) {
            Some(config_path) => {
                tracing::info!("Configuration path found: {}", config_path.display());
                Some(config_path)
            }
            None => None,
        },
    };
    let config_rules = match &config_path {
        Some(path) => match load_config(path) {
            Ok(config) => {
                let rules = config.rules().into_iter().collect::<Vec<_>>();
                tracing::info!("Loaded config: {} with {} rules", path.display(), rules.len());
                rules
            }
            Err(why) => {
                tracing::error!("Error loading config: {why}");
                SemRelConfig::default().rules().into_iter().collect::<Vec<_>>()
            }
        },
        None => {
            tracing::info!("Using default rules for configuration.");
            SemRelConfig::default().rules().into_iter().collect::<Vec<_>>()
        }
    };
    let rules = parse_rules(&opts.rule)?
        .chain(config_rules)
        .chain(build_default_rules())
        .collect::<Vec<_>>();
    tracing::info!("Active rules: {}", rules.len());
    for (commit_type, bump_rule) in rules.iter() {
        tracing::trace!(" - Active: {commit_type:?} -> {bump_rule:?}");
    }
    let manifest_path = find_manifest(path)?;
    let changelog = get_changelog(&repo, &manifest_path, &rules)?;
    tracing::info!("Found manifest: {}", manifest_path.display());
    let current_version = changelog.current_version;
    tracing::info!("Found manifest version: {current_version}");
    let bump = opts.bump.unwrap_or_default();
    tracing::info!("Found bump rule: {bump}");
    let new_version = match bump {
        BumpRule::Notset => changelog.next_version(&rules),
        _ => changelog.current_version.bump(bump),
    };
    tracing::info!("Calculated new version: {new_version}");

    let cli_data = CliData {
        manifest_path,
        rules: rules.to_vec(),
        config_path,
        changelog,
        new_version,
        current_version,
    };

    match opts.cmd {
        Command::Update => handle_update(&cli_data),
        Command::Show { cmd } => handle_show_command(cmd, &cli_data),
        Command::Config { cmd } => handle_config_command(cmd, &cli_data),
    }
}

fn handle_update(cli_data: &CliData) -> anyhow::Result<()> {
    let manifest_data = std::fs::read(&cli_data.manifest_path)?;
    let data = String::from_utf8(manifest_data)?;
    let mut supported_manifest = SupportedManifest::parse(&cli_data.manifest_path, data)?;
    supported_manifest.set_version(cli_data.new_version)?;
    supported_manifest.write(&cli_data.manifest_path)?;
    println!("Wrote to: {}", cli_data.manifest_path.display());
    Ok(())
}
fn handle_config_command(cmd: ConfigOpts, cli_data: &CliData) -> anyhow::Result<()> {
    match cmd {
        ConfigOpts::Edit => {
            let config_path = match &cli_data.config_path {
                Some(path) => path.to_owned(),
                None => {
                    let manifest_path = cli_data.manifest_path.as_path().parent().unwrap();
                    manifest_path.join(DEFAULT_CONFIG_FILENAME)
                }
            };
            // If the file does not exist, let's preseed this with the rules we've captured already
            match !config_path.exists() {
                true => {
                    let mut default_config = SemRelConfig::default();
                    default_config.extend_rules(&cli_data.rules);
                    let toml = toml::to_string(&default_config).unwrap();
                    std::fs::write(&config_path, toml).expect("Unable to write file");
                }
                false => match load_config(&config_path) {
                    Ok(config) => {
                        if !config.has_rules() {
                            let mut default_config = SemRelConfig::default();
                            default_config.extend_rules(&cli_data.rules);
                            let toml = toml::to_string(&default_config).unwrap();
                            std::fs::write(&config_path, toml).expect("Unable to write file");
                        }
                    }
                    Err(_) => {
                        let mut default_config = SemRelConfig::default();
                        default_config.extend_rules(&cli_data.rules);
                        let toml = toml::to_string(&default_config).unwrap();
                        std::fs::write(&config_path, toml).expect("Unable to write file");
                    }
                },
            }
            // Interactively run the default editor if it is set
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
            let status = std::process::Command::new(editor).arg(config_path).status()?;
            if !status.success() {
                return Err(anyhow::anyhow!("Failed to open editor. Please set `EDITOR` environment variable to your preferred editor."));
            }
            Ok(())
        }
    }
}

fn handle_show_command(cmd: ShowOpts, cli_data: &CliData) -> anyhow::Result<()> {
    match cmd {
        ShowOpts::Config => {
            if cli_data.config_path.is_none() {
                eprintln!("No configuration file found.");
                std::process::exit(1);
            } else {
                let config_path = cli_data.config_path.clone().unwrap();
                let config = load_config(config_path).unwrap_or_default();
                let rules = config.rules().into_iter().collect::<Vec<_>>();
                if rules.is_empty() {
                    eprintln!("No configuration data found");
                    std::process::exit(1);
                } else {
                    let mut shown = vec![];
                    for (commit_type, bump_rule) in rules {
                        if shown.contains(&commit_type) {
                            continue;
                        }
                        shown.push(commit_type.to_owned());
                        if bump_rule != BumpRule::NoBump {
                            println!("{:?} -> {:?}", commit_type, bump_rule);
                        }
                    }
                }
            }
            Ok(())
        }
        ShowOpts::Rules => {
            let mut shown = vec![];
            for (commit_type, bump_rule) in &cli_data.rules {
                if shown.contains(commit_type) {
                    continue;
                }
                shown.push(commit_type.to_owned());
                if *bump_rule != BumpRule::NoBump {
                    println!("{:?} -> {:?}", commit_type, bump_rule);
                }
            }
            Ok(())
        }
        ShowOpts::Manifest => {
            println!("{}", cli_data.manifest_path.display());
            Ok(())
        }
        ShowOpts::Notes => {
            println!("{}", cli_data.changelog.release_notes(&cli_data.rules));
            Ok(())
        }
        ShowOpts::Log => {
            for item in &cli_data.changelog.changes {
                println!("{} {}", item.id, item.message())
            }
            Ok(())
        }
        ShowOpts::Next => {
            println!("{}", cli_data.new_version);
            Ok(())
        }
        ShowOpts::Current => {
            println!("{}", cli_data.current_version);
            Ok(())
        }
    }
}
