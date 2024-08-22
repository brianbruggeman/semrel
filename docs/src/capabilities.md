# Capabilities

The following capabilities are supported by the current version of the tool:

- **Conventional Commits**: The tool uses Conventional Commits to determine the next version. This is inspired by semantic-release.
- **Version determination**: The tool determines the next version based on the commit history from a git repository.
- **Logging**: The tool logs the versioning decisions made during the process.  This can be accessed by setting `RUST_LOG`
- **Standalone**: The tool is a standalone binary that requires no additional plugins or packages.
- **Compatibility**: The tool is compatible with various languages and ecosystems.
- **Customizable**: The tool allows for custom rules to be defined for commit types.