use clap::Parser;
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
    #[clap(short, long)]
    bump: Option<BumpRule>,
    /// Update the manifest
    #[clap(long)]
    update: bool,
    /// Custom rules for commit types (can be comma separated)
    #[clap(short, long, num_args(0..))]
    rule: Vec<String>,

    /// Show only the current version
    #[clap(long)]
    current: bool,
    /// Show commit log used to calculate the version
    #[clap(long)]
    log: bool,
    /// Show release notes
    #[clap(long)]
    notes: bool,
    /// Show the path to the manifest file
    #[clap(long)]
    manifest: bool,
    /// Show the rules
    #[clap(long)]
    rules: bool,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();
    let opts = Opts::parse();

    let path = &opts.path;
    let repo = get_repo(path).map_err(|_| RepositoryError::InvalidRepositoryPath(path.into()))?;
    let rules = parse_rules(&opts.rule)?
        .chain(build_default_rules())
        .collect::<Vec<_>>();
    let changelog = get_changelog(&repo, &rules)?;
    let current_version = changelog.current_version;
    let bump = opts.bump.unwrap_or_default();
    let new_version = match bump {
        BumpRule::Notset => {
            changelog.next_version(&rules)
        }
        _ => {
            changelog.current_version.bump(bump)
        }
    };

    if opts.rules {
        for (ct, br) in rules {
            println!("{}: {:?}", ct, br);
        }
        return Ok(());
    }

    if opts.current {
        println!("{current_version}");
        return Ok(());
    }

    if opts.notes {
        println!("{}", changelog.release_notes(&rules));
        return Ok(());
    }

    if opts.manifest {
        let manifest_path = find_manifest(path).map_err(|_| RepositoryError::InvalidManifestPath(path.into()))?;
        println!("{}", manifest_path.display());
        return Ok(());
    }

    if opts.log {
        for commit_info in changelog.changes {
            println!("{} {}", commit_info.id, commit_info.message());
        }
        return Ok(());
    }

    if opts.update {
        let manifest_path = find_manifest(path).map_err(|_| RepositoryError::InvalidManifestPath(path.into()))?;
        let data = std::fs::read_to_string(&manifest_path)?;
        let mut manifest = SupportedManifest::parse(&manifest_path, data)?;
        manifest.set_version(new_version)?;
        manifest.write(&manifest_path)?;
        return Ok(());
    }

    println!("{new_version}");
    Ok(())
}