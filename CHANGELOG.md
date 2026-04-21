# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-04-21

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

### Security

- All CI workflows hardened per supply-chain playbook: actions pinned to SHA,
  least-privilege permissions, persist-credentials disabled, cargo-deny +
  poutine + zizmor audits
- Build provenance attestation on release artifacts

[Unreleased]: https://github.com/graelo/git-perms/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/graelo/git-perms/releases/tag/v0.1.0
