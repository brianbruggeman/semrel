# Motivation

## Problem

The current deployment process in many projects is only partially automated, with versions being managed manually within pull requests. This often leads to versions being forgotten to be updated, causing confusion about the deployed version and slowing down the deployment process. The team needs a lightweight, customizable tool that integrates well with git and command-line interfaces, and can determine the next version based on commit history. The frequency of forgotten version updates, the process for backtracking when a version is forgotten, the checks in place for version updates, and the process for tracking the version in production are all areas of concern. The impact of the current process on productivity and the specific issues arising from forgotten version updates also need to be addressed.

## Existing Solutions

We looked at semantic-release, which is a popular tool for automating versioning and releases. Cargo-release is another tool, but in both cases, they seem to be primarily focused on a specific domain (e.g. npm packages or Rust projects).  The expansive ecosystem of semantic-release is compelling.  But the tooling itself was difficult to debug and failed to just work in Rust and Python projects.  Because we want to support Rust, Python, Typescript and Javascript, we needed something that was lightweight, more flexible and plugged into existing github actions without obfuscating the process used to develop the next version.

## Goals

- Implement Conventional Commits for version determination, inspired by semantic-release.
- Establish a clear and intuitive process for versioning.
- Enhance user understanding through comprehensive logging of versioning decisions.
- Develop a standalone tool that requires no additional plugins or packages.
- Ensure the tool's compatibility across various languages and ecosystems.
