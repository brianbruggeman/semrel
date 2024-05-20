# semrel

A semantic release tool

## Motivation

After spending way too much time attempting to figure out the perfect combination of configuration for semantic-release using cargo, I'm just going to go ahead and write my own.

The goal for this project is to produce a really simple, ergonomic interface for identifying the next version based on git commits.  When semrel bumps a version it will
create an annotated git tag along with a signed commit.

## Usage

### Simple usage
```bash
cd /path/to/your/repo
semrel
```

### Using a pipe to pass the commit message
```bash
cd /path/to/your/repo
git log -1 --pretty=%B | semrel
```

### Passing the commit message directly
```bash
semrel "feat: this is a new feature"
```

### Controlling which path to use
```bash
semrel -p/--path /path/to/your/repo
```

### Explicitly forcing a bump
```bash
semrel -b/--bump <major|minor|patch>
```

### Displaying the current version
```bash
semrel -c/--current
```

## Installation

```bash
cargo install --git https://api.github.com/repos/brianbruggeman/semrel
```


## Basic rules

- If the commit message contains "feat:", then the minor version is bumped.
- If the commit message contains "fix:" or "chore:", then the patch version is bumped.
- If the commit message contains "BREAKING CHANGE:", then the major version is bumped.
- Everything else is ignored.


## Other notes

- Git commands are used through [gix](https://docs.rs/gix/0.62.0/gix/).
- Support was added for:
    - Python: pyproject.toml (pep621 and poetry)
    - Javascript/Typescript: package.json
    - Rust: Cargo.toml
- It is possible to extend the support outside of the crate for other formats using the [Manifest](src/core/manifest.rs)
- Example signed git commit:
```
git commit -S -m "{commit_message}"
```
- Example annotated git tag:
```
git tag -a {tag_name} -m "{tag_message}"
```

## Future work

In no particular order, here's ways to improve the status quo
- [x] Control the next version with --bump=patch|minor|major
- [x] Display the current version with --current
- [x] Create a specific bump rule with --rule=<prefix>=patch|minor|major to create a custom rule
- [ ] Update the manifest file with --update
- [ ] Create a commit with --commit
- [ ] Create a tag with --tag
- [ ] Search through the commit history to find the last version bump
- [ ] Search through the commit history to collect all of the previous commits since the last version bump
- [ ] Create a signed commit with --signed
- [ ] Create an annotated tag with --annotate
- [ ] Create a configuration file and allow people to customize the rules