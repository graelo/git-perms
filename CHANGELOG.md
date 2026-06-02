# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.2] - 2026-06-02

### Added

- `git perms save` — scan tracked files and write permissions to `.gitperms`
- `git perms restore` — read `.gitperms` and apply file modes
- `git perms diff [--quiet]` — show or detect mode differences
- `git perms hook install` / `uninstall` — manage git hooks (pre-commit,
  post-checkout, post-merge, post-rewrite) for automatic save/restore
- `git perms generate-completion <shell>` — shell completions for bash, zsh,
  fish
- Configurable pre-commit behavior via `git config perms.preCommit`
  (`auto`, `warn`, `block`)

### Changed

- Updated Cargo dependencies (minor and patch releases)

### Security

- All CI workflows hardened per supply-chain playbook: actions pinned to SHA,
  least-privilege permissions, persist-credentials disabled, cargo-deny +
  poutine + zizmor audits
- Build provenance attestation on release artifacts
- CI workflows aligned with supply-chain playbook v1.1; Renovate granted
  `create-github-app-token` permissions

[Unreleased]: https://github.com/graelo/git-perms/compare/v0.0.2...HEAD
[0.0.2]: https://github.com/graelo/git-perms/compare/v0.0.1...v0.0.2
