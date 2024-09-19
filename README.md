[![CI Workflow](https://github.com/brianbruggeman/semrel/actions/workflows/ci.yml/badge.svg)](https://github.com/brianbruggeman/semrel/actions/workflows/ci.yml)
[![Release Workflow](https://github.com/brianbruggeman/semrel/actions/workflows/release.yml/badge.svg)](https://github.com/brianbruggeman/semrel/actions/workflows/release.yml)
[![Audit Workflow](https://github.com/brianbruggeman/semrel/actions/workflows/audit.yml/badge.svg)](https://github.com/brianbruggeman/semrel/actions/workflows/audit.yml)


# semrel
A semantic release tool

## Docs

See the [book](./docs/src/SUMMARY.md) for more information.

### Configuration

While the configuration _is_ [detailed]((./docs/src/configuration.md)) in the book, here is a quick reference for creating or updating the semrel config file at the project level or the root of the repository (e.g. `.semrel.toml`), or under an XDG compliant path (e.g. `~/.config/semrel/config.toml`) or for the system under `/etc/semrel/config.toml`:

```toml
[semrel.rules]
feat = "minor"
chore = "patch"
fix = "patch"
perf = "patch"
refactor = "patch"
revert = "patch"
style = "patch"
build = "none"
ci = "none"
cd = "none"
docs = "none"
test = "none"
```
## Usage

## Github action

The github action will install semrel into the current (`.`) path for your github action and can be used subsequently in any run step.  Additionally, the following are output:

- `current-version`:  this will represent the current version found within the manifest
- `next-version`: this will represent the calculated next version from the `current-version`
- `log`: this is a base-64 encoded form of the log lines used to generate the next release version
- `release-notes`: this is a base-64 encoded form of the release notes based on git log parsed
- `version-changed`: this boolean identifies a version change

This should be all you need to add.  Note that you need to fetch the full history for the repository, otherwise the git log will not be able to find the commits and semrel will not find the previous version.

```yaml
- uses: actions/checkout@v4
  with:
    fetch-depth: 0

- name: Run semrel
  id: semrel
  uses: brianbruggeman/semrel@main
```

To use, then:
```yaml
- name: Take some action
  if: ${{ steps.semrel.outputs.version-changed }}
  run: |
      echo ${{ steps.semrel.outputs.release-notes }} | base64 --decode > SEMREL_RELEASE_NOTES.md
      echo ${{ steps.semrel.outputs.log }} | base64 --decode > SEMREL_LOG.md
      # update the current manifest
      ./semrel update
      # Take more steps here to check in the manifest file and/or create a release

- name: Create Release Notes
  run: printf "%s" "${{ needs.semrel.outputs.release-notes }}" | base64 --decode > release-notes-${{ needs.semrel.outputs.next-version }}.md
```

In CI/CD, you _can_ ask semrel to checkout the full depth.  But there are some edge cases:

    - When you checkout it removes the local filesystem; any changes you made locally will be lost.
    - By default, semrel does not checkout any files.
    - Generally this is most useful at the start of a workflow, but you can use it at any time.

```yaml
- name: Run semrel
  id: semrel
  uses: brianbruggeman/semrel@main
  with:
    branch: ${{ github.head_ref || github.ref_name }}
```

If you have a subproject or follow a monorepo structure, you may want to control which
path is searched for updates.  You can specify the path.  Semrel expects that the path
contains a manifest file (e.g. Cargo.toml, package.json, pyproject.toml, etc.):

```yaml
- name: Run semrel
  id: semrel
  uses: brianbruggeman/semrel@main
  with:
    path: './to/some/sub-project'
```

### Command-line

```bash
$ semrel
Usage: semrel [OPTIONS] <COMMAND>

Commands:
  update  Update the manifest
  show    Show information
  config  Config subcommand
  help    Print this message or the help of the given subcommand(s)

Options:
      --path <PATH>                Path to the project root [env: PROJECT_PATH=] [default: .]
      --rule [<RULE>...]           Custom rules for commit types (can be comma separated) [env: SEMREL_RULES=]
  -b, --bump <BUMP>                Short circuit for bumping the version [env: SEMREL_BUMP=] [possible values: major, minor, patch, none]
      --config-path <CONFIG_PATH>  Specify the configuration path [env: SEMREL_CONFIG_PATH=]
  -h, --help                       Print help
```

```bash
$ semrel show
Show information

Usage: semrel show [OPTIONS] <COMMAND>

Commands:
  current         Show only the current version
  next            Show the next version
  log             Show the changelog
  notes           Show the release notes
  manifest        Show the manifest path
  rules           Show the rules
  config          Show the configuration
  release-commit  Show suggested release commit message
  help            Print this message or the help of the given subcommand(s)

Options:
      --path <PATH>                Path to the project root [env: PROJECT_PATH=] [default: .]
      --rule [<RULE>...]           Custom rules for commit types (can be comma separated) [env: SEMREL_RULES=]
  -b, --bump <BUMP>                Short circuit for bumping the version [env: SEMREL_BUMP=] [possible values: major, minor, patch, none]
      --config-path <CONFIG_PATH>  Specify the configuration path [env: SEMREL_CONFIG_PATH=]
  -h, --help                       Print help
```

```bash
$ semrel config
Config subcommand

Usage: semrel config [OPTIONS] <COMMAND>

Commands:
  edit  Edit the current configuration
  help  Print this message or the help of the given subcommand(s)

Options:
      --path <PATH>                Path to the project root [env: PROJECT_PATH=] [default: .]
      --rule [<RULE>...]           Custom rules for commit types (can be comma separated) [env: SEMREL_RULES=]
  -b, --bump <BUMP>                Short circuit for bumping the version [env: SEMREL_BUMP=] [possible values: major, minor, patch, none]
      --config-path <CONFIG_PATH>  Specify the configuration path [env: SEMREL_CONFIG_PATH=]
  -h, --help                       Print help
```


## Installation

There are no dependencies for this project.  You should be able to simply download the binary on the release page and run it.

However, if you want to build it yourself, you can use the following command:

```bash
cargo install --git https://api.github.com/repos/Wizehire/semrel
```
