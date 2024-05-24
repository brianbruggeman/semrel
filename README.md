# semrel

A semantic release tool

## Motivation

I wanted a simple tool that did the right thing in figuring out how to bump the version.  I looked at semantic-release and took inspiration from that project, however I wanted to reduce the complexity and make it easier for me to use.

## Usage

```bash
$ semrel --help

Usage: semrel [OPTIONS]

Options:
  -p, --path <PATH>       Path to the project root [default: .]
  -b, --bump <BUMP>       Force a bump [possible values: major, minor, patch, none]
      --update            Update the manifest
  -r, --rule [<RULE>...]  Custom rules for commit types (can be comma separated)
      --current           Show only the current version
      --log               Show commit log used to calculate the version
      --notes             Show release notes
      --manifest          Show the path to the manifest file
      --rules             Show the rules
  -h, --help              Print help
```

## Installation

```bash
cargo install --git https://api.github.com/repos/brianbruggeman/semrel
```


## Basic rules

The rules can be accessed by using the `--rules` flag.

### Major

To update a major version, the commit message must contain "BREAKING CHANGE".  Alternatively, you can force a major version bump by using the `--bump=major` flag.

### Minor

Minor versions are updated when the commit message contains "feat".  Alternatively, you can force a minor version bump by using the `--bump=minor` flag.

### Patch

Patch versions are updated when the commit message contains "fix", "chore", "perf", "refactor", "revert", "style".  Alternatively, you can force a patch version bump by using the `--bump=patch` flag.



## Notes

There is a significant amount of logging I've added to make it clear on how the version is being calculated.  This is useful for debugging and understanding how the version is being calculated.  To enable, just use `RUST_LOG=debug` in the environment when running.

```bash
RUST_LOG=debug semrel
```

Support was added for the following:
- Python: pyproject.toml (pep621 and poetry)
- Javascript/Typescript: package.json
- Rust: Cargo.toml

This can be further extended to support other languages and manifest files.

## Future work

- [ ] Generating commits, tags, etc.
- [ ] Create a configuration file and allow people to customize the rules
