# Release notes: 0.3.0 (2024-05-25)


## Features

### docker
- Add example dockerfile
- add configuration support
- added config support


## Fixes

### deps
- reduce git footprint

### commit-type
- update release notes
- adds missing update functionality
- add missing config

### changelog
- manages edge cases around bumps


## Refactor

### cli
- moved options to subcommands


## Style
- update formatting


## Test

### invalid_json
- fix broken test


## Chore

### git
- fix ignored

### changelog
- renamed log.rs to changelog.rs
- update cargo

### env
- add dotenv support


## Continuous Integration

### bump
- fix the push
- adjust triggers
- add missing .cargo
- improve compile times
- add caching
- control nightly better
- fix rustfmt
- use rustup instead of a github action
- also include clippy and fmt for nightly
- try forcing nightly
- fixes missing triggers
- add ci


## Deployment

### xcompile
- add archive

### native
- cleanup musl step
- fix build target
- fix extra character
- specify the target
- need to use the next version
- only run on branch main after CI has completed
- finalize cd
- update bump with new cli
- drop extra workflow stuff
- finalize cd
- fix cli calls
- fix binary permissions
- fix sequence of steps
- actually update Create release instead of Upload...
- update token for release
- add token for release
- update release assets and process
- try using cross for all musl architectures
- maybe cross will just work?
- only install cross if uncached
- build single static binaries using musl
- add linking directive for xcompiling
- add specific steps for gnu gcc arm
- add caching to reduce compilation times
- fix xcompile command
- split native and cross compiling builds
- add os to matrix
- fix matrix variable
- fix matrix variable

### release
- fix os

### archive
- greatly reduce artifacts stored

### bump
- build everything
- use the correct files
- allow release to write to repo
- use semrel on path
- put the right semrel in .cargo/bin
- put semrel in .cargo/bin
- fix cost redux change
- reduce cost
- what path can I use
- adjust semrel artifact names
- adjust semrel location
- adjust semrel location and name and sequence
- remove bump-release workflow
- update version within release step
- fetch the entire history
- fetch the entire history
- fetch the history
- give more debug info
- move update to catch it in the commmit
- modify fetch and rebase
- drop duplicate token
- fix permissions
- fix secrets
- fix push path
- fix path
- add missing push again
- add caching to bump


## Documentation
- adds new cli interface back into docs
- simplify readme


## release
- 0.2.0


## Non Compliant
- Update caching for bump version workflow
- Update bump/cd/ci interactions
- tested release

