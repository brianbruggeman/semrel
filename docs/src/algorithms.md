# Algorithm

## Overview

The algorithm navigates through a repository's commit history, using Conventional Commits to determine the next version and generate release notes. It's designed for use in CI/CD pipelines, providing automated version determination and release notes creation.

Starting with the current version from the manifest file, the algorithm backtracks through the commits in the current branch, categorizing commit messages according to Conventional Commits. It identifies the highest level bump and the last updated manifest file at that level. If a commit type above the current level is found, the search continues at this new level until a manifest file with the correct version update is found, at which point the new version is returned.

A PEST parser is used to parse commit messages and extract the commit type, scope, subject, body, and footer. Non-compliant commit messages are recognized but do not trigger a version bump by default. Custom rules for version bumping can be provided, with the algorithm prioritizing rules from the command-line --rule flag, the configuration file, and finally, hard-coded default rules.

## Changelog

The tool will first look at the current manifest file and determine the current version.  Then the tool will start walking backwards for all of the commits in the current branch.  The tool looks for and categorizes the commit messages based on Conventional Commits.  The following are recognized commit types:

- `build`: Changes that affect the build system or external dependencies (example scopes: cargo, npm)
- `cd`: Continuous Deployment
- `chore`: Other changes that don't modify src or test files
- `ci`: Changes to our CI configuration files and scripts (example scopes: github-actions)
- `docs`: Documentation only changes
- `feat`: A new feature
- `fix`: A bug fix
- `perf`: A code change that improves performance
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `revert`: Reverts a previous commit
- `style`: Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc)
- `test`: Adding missing tests or correcting existing tests

As the tool is walking, the tool is checking for the highest level bump and the last updated manifest file that has the same level.  When the tool identifies a commit type above the current level, then the tool will continue searching for that new level.  Once the tool identifies a manifest file with the correct version update for the new level, the tool will stop searching and return the new version.

## Base Commit bump rules

To extract that information, a PEST parser is used to parse the commit messages.  The parser is defined in the `src/core/conventional_commits/commit_message.pest` file.  The parser is then used to extract the commit type, scope, subject, body, and footer.  The commit type is used to determine the next version.  The scope and description are used to generate the release notes.

To be compliant, the type is required, but the scope, body, and footer are optional.  The type is used to determine the next version.  The scope is used to group the commits in the release notes.  The body and footer are used to generate the release notes.

Additionally, non-compliant commit messages are recognized, but will not bump the version by default.  It is possible to generally follow the conventional commit format and then provide custom rules for the bumping of the version.

Default rules are encoded in the `src/core/semantic_release/bump_rule_mapping.rs` file.

- Major: `BREAKING CHANGE` must be present somewhere in the commit message
- Minor: `feat` is the only commit type by default that bumps a minor version
- Patch: `chore`, `fix`, `perf`, `refactor`, `revert`, and `style` will all bump a patch version

> **NOTE:** The `BREAKING CHANGE` must be in the commit message body or footer.  It is not required to be in the commit message subject.
> **NOTE:** the scope cannot be a standard commit type.  If the scope is a standard commit type, then the tool will flag it as an error.

### Examples

#### Template
The basic format of a commit message is:

```
<type>(<scope>): <subject>
<BLANK LINE>
<body>
<BLANK LINE>
<footer>
```

#### Breaking Change (Major)
A full example of a major breaking change. In this example, the type is `feat`, the scope is `core`, and the subject is `add new feature`.
```feat(core): add new feature

This is a new feature that we have added to the core library.

BREAKING CHANGE: The function signature has changed from `fn foo(a: i32, b: i32) -> i32` to `fn foo(a: i32, b: i32, c: i32) -> i32`.  This was necessary to support building our new feature.
```

#### Feature update (Minor)
A smaller example of a minor feature.  In this example, the type is `feat`, the scope is `core`, and the subject is `add new feature`.
```feat(core): add new feature```

#### Bug Fix (Patch)
A smaller example of a patch fix.  In this example, the type is `fix`, the scope is `core`, and the subject is `fix bug`.
```fix(core): fix bug```

#### Semi-Compliant (NoBump)
A semi-compliant example.  In this semi-compliant example, the type is `ENG-1234`, the scope is `none`, and the subject is `fix bug`.  Because the type is not recognized, the tool will not bump the version.
```ENG-1234: fix bug```

#### Non-Compliant (NoBump)
A Non-compliant example.  In this non compliant example, there is no type.  The entire message is pulled into the subject.
```JohnDoe - add new feature```


## Customzing Rules

Custom rules currently can be added via a --rule flag.  The algorithm for determining the bump rule to use is based on "which comes first".  The order of precedence is:

- command-line --rule flag
- configuration file
- hard-coded default rules

So if a rule is present in the command-line --rule flag, then that rule will be used.  If a rule is not present in the command-line --rule flag, then the configuration file will be checked.  If a rule is not present in the configuration file, then the hard-coded default rules will be used.

The bump rules are a critical aspect because the version is determined by the highest level bump found in the commit history.  If rules are changed between runs, then the version will be different.

An example for a custom rule might be:

```bash
$ semrel --rule ENG-1234=minor
```

And then with the following commit message, we would bump the minor version:
```ENG-1234: add new feature```
