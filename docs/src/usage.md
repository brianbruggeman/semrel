# Usage

This section contains a few examples of how to use the library from the command line.

## How we might use this
```bash
# What's the change log for this new version?
$ semrel show log
f7e8fc4fc26f83f8053bd78c9475ba67b643f182 ci: add ci
ec79645b01ee77c6a7fa9b522534fbcf159c22ea chore(git): fix ignored
191ba409562f345d290014fa2a1c30fb784c61b7 feat(docker): Add example dockerfile
d47b19bbd040725d37d30f155e0ed78088cc1895 docs: simplify readme
c78f934b20845c0f1dfdd95043eb904d94fbb29a fix(commit-type): update release notes
06b31c48954e0b48599cf78d179586f69277c398 fix(deps): reduce git footprint
078e0ec70e6a0699e0654d09320a5808c2acfaad test(invalid_json): fix broken test
97f72245da66f1eef77ff949c1935f4e08fbaa98 fix(changelog): manages edge cases around bumps
3f2e48af7ddc2fe38de15b710d07b8747c0971a3 tested release

# What's the current version?
$ semrel show current
0.2.0

# What's the next version based on the changelog?
$ semrel show next
0.3.0

# How did we get to this version?
$ RUST_LOG=semrel=debug semrel show next
2024-05-24T12:39:42.266931Z DEBUG semrel::core::git::repo: Searching for repository under: .
2024-05-24T12:39:42.267995Z DEBUG semrel::core::git::repo: Found repository at: /Users/brianbruggeman/repos/mine/semrel
2024-05-24T12:39:42.268012Z DEBUG semrel::core::git::log: Starting get_changelog for path: /Users/brianbruggeman/repos/mine/semrel
2024-05-24T12:39:42.268054Z DEBUG semrel::manifests::supported_manifests: Parsing manifest: "/Users/brianbruggeman/repos/mine/semrel/Cargo.toml"
2024-05-24T12:39:42.268062Z DEBUG semrel::manifests::cargo_toml: Parsing Cargo.toml
2024-05-24T12:39:42.268293Z DEBUG semrel::manifests::cargo_toml: Parsed manifest.
2024-05-24T12:39:42.268308Z DEBUG semrel::manifests::supported_manifests: Getting version from manifest
2024-05-24T12:39:42.268312Z DEBUG semrel::manifests::supported_manifests: Version: Ok(SimpleVersion { major: 0, minor: 2, patch: 0 })
2024-05-24T12:39:42.268316Z DEBUG semrel::manifests::supported_manifests: Parsed manifest version: SimpleVersion { major: 0, minor: 2, patch: 0 }
2024-05-24T12:39:42.268320Z DEBUG semrel::manifests::supported_manifests: Getting filename from manifest
2024-05-24T12:39:42.268322Z DEBUG semrel::manifests::supported_manifests: Filename: Ok("Cargo.toml")
...
2024-05-24T12:39:42.284358Z DEBUG semrel::core::git::log: Finished get_changelog
2024-05-24T12:39:42.284368Z DEBUG semrel::core::semantic_release::bump_rule_mapping: Finding rule for Ci
2024-05-24T12:39:42.284371Z DEBUG semrel::core::semantic_release::bump_rule_mapping: Finding rule for Chore
2024-05-24T12:39:42.284373Z DEBUG semrel::core::semantic_release::bump_rule_mapping: Finding rule for Feat
2024-05-24T12:39:42.284375Z DEBUG semrel::core::semantic_release::bump_rule_mapping: Finding rule for Docs
2024-05-24T12:39:42.284378Z DEBUG semrel::core::semantic_release::bump_rule_mapping: Finding rule for Fix
2024-05-24T12:39:42.284382Z DEBUG semrel::core::semantic_release::bump_rule_mapping: Finding rule for Fix
2024-05-24T12:39:42.284384Z DEBUG semrel::core::semantic_release::bump_rule_mapping: Finding rule for Test
2024-05-24T12:39:42.284386Z DEBUG semrel::core::semantic_release::bump_rule_mapping: Finding rule for Fix
2024-05-24T12:39:42.284388Z DEBUG semrel::core::semantic_release::bump_rule_mapping: Finding rule for NonCompliant
0.3.0
```

## Basic usage

The following command will determine the next version based on the commit history of the current repository:

```bash
$ semrel
0.3.0
```

## Show the current version
```bash
$ semrel show current
0.2.3
```

## Custom rules

Custom rules can be defined via the `--rule` flag.  The following command will bump the version to a minor version if the commit message contains `ENG-1234`:

```bash
$ git log -1 --pretty=%B
ENG-1234: add new feature

$ semrel show next --rule ENG-1234=minor
0.3.0
```

# Update the manifest
```bash
$ semrel update
```

## Show the log used to calculate the version

The following command will show the commit log used to calculate the version:

```bash
$ semrel show log
f7e8fc4fc26f83f8053bd78c9475ba67b643f182 ci: add ci
ec79645b01ee77c6a7fa9b522534fbcf159c22ea chore(git): fix ignored
191ba409562f345d290014fa2a1c30fb784c61b7 feat(docker): Add example dockerfile
d47b19bbd040725d37d30f155e0ed78088cc1895 docs: simplify readme
c78f934b20845c0f1dfdd95043eb904d94fbb29a fix(commit-type): update release notes
06b31c48954e0b48599cf78d179586f69277c398 fix(deps): reduce git footprint
078e0ec70e6a0699e0654d09320a5808c2acfaad test(invalid_json): fix broken test
97f72245da66f1eef77ff949c1935f4e08fbaa98 fix(changelog): manages edge cases around bumps
3f2e48af7ddc2fe38de15b710d07b8747c0971a3 tested release
```

## Show the release notes

The following command will show the release notes:

```bash
$ semrel show notes
# Release notes: 0.3.0 (2024-05-24)


## Features

### docker
- Add example dockerfile


## Fixes

### changelog
- manages edge cases around bumps

### commit-type
- update release notes

### deps
- reduce git footprint


## Test

### invalid_json
- fix broken test


## Chore

### git
- fix ignored


## Continuous Integration
- add ci


## Documentation
- simplify readme


## Non Compliant
- tested release
```