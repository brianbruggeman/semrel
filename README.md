# semrel

A semantic release tool

## Docs

See the [book](./docs/src/SUMMARY.md) for more information.

## Usage

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
  current   Show only the current version
  next      Show the next version
  log       Show the changelog
  notes     Show the release notes
  manifest  Show the manifest path
  rules     Show the rules
  config    Show the configuration
  help      Print this message or the help of the given subcommand(s)

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