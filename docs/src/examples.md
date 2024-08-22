# Examples

This section contains a few examples of how to use the library from the command line.

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

## Show the log used to calculate the version

The following command will show the commit log used to calculate the version:

```bash
$ semrel --log
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
$ semrel --notes
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