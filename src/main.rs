use atty::Stream;
use clap::{CommandFactory, Parser};
use semrel::*;
use std::io::{self, Read};

#[derive(Parser)]
pub struct Opts {
    /// The commit message to parse
    #[clap(default_value = "")]
    commit_message: String,

    /// Path to the project root
    #[clap(short, long, default_value = ".")]
    path: String,
}

fn main() {
    let opts = Opts::parse();
    let mut commit_message = opts.commit_message.clone();

    if commit_message.is_empty() {
        if atty::is(Stream::Stdin) {
            Opts::command().print_help().unwrap();
            return;
        } else {
            let mut input = String::new();
            io::stdin().read_to_string(&mut input).unwrap();
            commit_message = input.trim().to_string();
        }
    }

    let manifest_path = find_manifest(opts.path).unwrap();

    let manifest = parse_manifest(manifest_path).unwrap();
    let commit = Commit::new(commit_message.clone()).unwrap();
    let old_version = manifest.version();
    let rule_used = get_rule(build_default_rules(), &commit.commit_type);
    let commit_type = commit.commit_type.clone();
    let new_version = bump_version(build_default_rules(), commit.commit_type, manifest.version());
    println!("{} => [{commit_type}] {rule_used}", commit_message);
    println!("{old_version} -> {new_version}");
}
