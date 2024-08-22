# Basic rules

## Major

The goal of a major version is to indicate a breaking change.  This can be done by including "BREAKING CHANGE" in the commit message body or footer.  Alternatively, you can force a major version bump by using the `--bump=major` flag. This will increment the major version by one and reset the minor and patch versions to zero.  For example, if the current version is `1.2.3`, the new version with a major bump will be `2.0.0`.  Keeping with best practices of Semantic Versioning, the major version should be updated when incompatible changes are introduced.

Breaking changes can include:
- Changes to the public API
- Changes to the configuration file
- Changes to the command-line interface
- Changes to the environment variables
- Changes to the database schema

A Breaking Change should have a detailed description of the change and the reasoning behind it.  This will help users understand the impact of the change and how to adapt to the new version.

## Minor

The goal of a minor version is to indicate a new feature.  This can be done by including "feat" in the commit message type.  Alternatively, you can force a minor version bump by using the `--bump=minor` flag. This will increment the minor version by one and reset the patch version to zero.  For example, if the current version is `1.2.3`, the new version with a minor bump will be `1.3.0`.  Keeping with best practices of Semantic Versioning, the minor version should be updated when new features are introduced.

New features can include:
- New functionality
- New endpoints
- New commands
- New configuration options
- New environment variables

A New Feature should have a detailed description of the feature and the reasoning behind it.  This will help users understand the new functionality and how to use it.  However, new features should not introduce breaking changes.  If a new feature requires a breaking change, it should be considered a major version bump.

## Patch

The goal of a patch version is to indicate a bug fix.  This can be done by including "fix" in the commit message type.  Alternatively, you can force a patch version bump by using the `--bump=patch` flag. This will increment the patch version by one.  For example, if the current version is `1.2.3`, the new version with a patch bump will be `1.2.4`.  Keeping with best practices of Semantic Versioning, the patch version should be updated when bugs are fixed or minor improvements are made.

Patches can include:
- Refactoring
- Performance improvements
- Code cleanup
- Reverting changes
- Style changes
- Bug fixes

The Bug Fix should have a detailed description of the bug and the reasoning behind the fix.  This will help users understand the impact of the bug and how to adapt to the new version.  However, bug fixes should not introduce breaking changes.  If a bug fix requires a breaking change, it should be considered a major version bump.

