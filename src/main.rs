use atty::Stream;
use clap::{CommandFactory, Parser};
use semrel::*;
use std::io::{self, Read};
use std::path::Path;

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
    let opts = Opts::parse();
    let commit_message = opts.commit_message.clone();
    let current_version = match get_current_version(&opts.path) {
        Ok(version) => version,
        Err(why) => {
            eprintln!("Failed to get current version: {why}");
            std::process::exit(1);
        }
    };
    if opts.current {
        println!("{current_version}");
        return Ok(());
    }
    let mut new_version = current_version.bump(opts.bump);
    if current_version == new_version {
        let commit = if commit_message.is_empty() {
            if atty::isnt(Stream::Stdin) {
                let mut input = String::new();
                io::stdin().read_to_string(&mut input).unwrap();
                match Commit::new(input.trim()) {
                    Ok(commit) => commit,
                    Err(why) => {
                        eprint!("Failed to parse commit message: {}", why);
                        return Ok(());
                    }
                }
            } else if is_repo(&opts.path) {
                match Commit::try_from(Path::new(&opts.path)) {
                    Ok(commit) => commit,
                    Err(_why) => {
                        Opts::command().print_help().unwrap();
                        std::process::exit(1);
                    }
                }
            } else {
                Opts::command().print_help().unwrap();
                std::process::exit(1);
            }
        } else {
            match Commit::new(commit_message.clone()) {
                Ok(commit) => commit,
                Err(why) => {
                    eprintln!("Failed to parse commit message: {}", why);
                    std::process::exit(1);
                }
            }
        };
        let rules = parse_rules(&opts.rule)?;
        let commit_type = commit.commit_type.clone();
        new_version = bump_version(rules, commit_type, current_version);
    }
    println!("{new_version}");
    Ok(())
}

pub fn get_current_version(path: impl AsRef<Path>) -> Result<SimpleVersion, ManifestError> {
    let manifest_path = find_manifest(path.as_ref())?;
    let manifest = parse_manifest(manifest_path)?;
    manifest.version()
}

fn parse_rules(rules: &[impl AsRef<str>]) -> anyhow::Result<impl Iterator<Item = (CommitType, BumpRule)> + '_> {
    let iter = rules
        .iter()
        .flat_map(|rule| rule.as_ref().split(','))
        .map(|rule| rule.split('=').take(2))
        .flat_map(|mut rule| {
            let commit_type = match &rule.next() {
                Some(commit_type) => CommitType::from(*commit_type),
                None => anyhow::bail!("No rule found."),
            };
            let bump_rule = match &rule.next() {
                Some(bump_rule) => BumpRule::from(*bump_rule),
                None => anyhow::bail!("Invalid rule for: {commit_type}"),
            };
            Ok((commit_type, bump_rule))
        });
    Ok(iter)
}
