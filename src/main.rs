use std::io::{self, Read};
use std::path::Path;

use atty::Stream;
use clap::{CommandFactory, Parser};
use tracing_subscriber::EnvFilter;

use semrel::*;

#[derive(Debug, Parser)]
pub struct Opts {
    /// The commit message to parse
    #[clap(default_value = "")]
    commit_message: String,

    /// Path to the project root
    #[clap(short, long, default_value = ".")]
    path: String,

    /// Force a bump
    /// - major, minor, patch, none
    #[clap(short, long, default_value = "notset")]
    bump: BumpRule,

    /// Show only the current version
    #[clap(short, long)]
    current: bool,

    /// Custom rules for commit types (can be comma separated)
    #[clap(short, long, num_args(0..))]
    rule: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    init_logging();
    let opts = Opts::parse();

    let current_version = get_current_version(&opts.path)?;
    if opts.current {
        show_current_version(&current_version);
        return Ok(());
    }

    let commit_message = get_commit_message(&opts)?;
    tracing::debug!("Parsing commit message: {commit_message:?}");
    let new_version = calculate_new_version(&opts, current_version, &commit_message)?;
    println!("{new_version}");

    Ok(())
}

fn calculate_new_version(opts: &Opts, current_version: SimpleVersion, commit_message: &str) -> Result<SimpleVersion, anyhow::Error> {
    let mut new_version = current_version.bump(opts.bump);

    if current_version == new_version {
        let commit = parse_commit_message(commit_message)?;
        let rules = parse_rules(&opts.rule)?;
        let commit_type = commit.commit_type.clone();
        new_version = bump_version(rules.chain(build_default_rules()), commit_type, current_version);
    }

    Ok(new_version)
}

fn get_commit_message(opts: &Opts) -> Result<String, anyhow::Error> {
    if !opts.commit_message.is_empty() {
        Ok(opts.commit_message.clone())
    } else if atty::isnt(Stream::Stdin) {
        read_commit_message_from_stdin()
    } else if is_repo(&opts.path) {
        get_commit_message_from_repo(&opts.path)
    } else {
        Opts::command().print_help().unwrap();
        std::process::exit(1);
    }
}

fn get_commit_message_from_repo(path: &str) -> Result<String, anyhow::Error> {
    Commit::try_from(Path::new(path))
        .map(|commit| commit.to_string())
        .map_err(|_why| {
            Opts::command().print_help().unwrap();
            std::process::exit(1);
        })
}

fn get_current_version(path: &str) -> Result<SimpleVersion, anyhow::Error> {
    get_current_version_from_manifest(path).map_err(|why| {
        eprintln!("Failed to get current version: {why}");
        std::process::exit(1);
    })
}

pub fn get_current_version_from_manifest(path: impl AsRef<Path>) -> Result<SimpleVersion, ManifestError> {
    let manifest_path = find_manifest(path.as_ref())?;
    let manifest = parse_manifest(manifest_path)?;
    manifest.version()
}

fn init_logging() {
    // Set up tracing with the `RUST_LOG` environment variable
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();
}

fn parse_commit_message(message: &str) -> Result<Commit, anyhow::Error> {
    Commit::new(message).map_err(|why| {
        eprintln!("Failed to parse commit message: {}", why);
        std::process::exit(1);
    })
}

fn read_commit_message_from_stdin() -> Result<String, anyhow::Error> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(input.trim().to_string())
}

fn show_current_version(version: &SimpleVersion) {
    println!("{version}");
}
