# Configuration

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

- next to the manifest file (e.g. Cargo.toml or package.json or pyproject.toml)
- at the root of the project (e.g. .semrel.toml)
- in an XDG compliant configuration directory (e.g. $XDG_CONFIG_HOME/semrel/config.toml)
- under $HOME/.config/semrel/config.toml (if $XDG_CONFIG_HOME is not set)
- in the system configuration directory (e.g. /etc/semrel/config.toml)
