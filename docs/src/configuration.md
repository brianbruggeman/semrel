# Configuration

## Command-line interace

To show the current configuration settings, run:

```bash
semrel show config
```

```note
Configurations are context aware based on the configurations present within the system, so running this within a project directory could be different than running within a repo.
```

## Bump Rules

The configuration file uses the Toml format. The following is an example of a configuration file and is the default configuration used by the tool.

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

## Location

The configuration file maybe located in the following locations (in order of precedence):

- next to the manifest file (e.g. path/to/project/`.semrel.toml`)
- at the root of the project (e.g. path/to/repo/.semrel.toml)
- in an XDG compliant configuration directory (e.g. $XDG_CONFIG_HOME/semrel/config.toml)
- under $HOME/.config/semrel/config.toml (if $XDG_CONFIG_HOME is not set)
- in the system configuration directory (e.g. /etc/semrel/config.toml)
