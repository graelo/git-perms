# AGENTS.md

This file contains instructions for coding agents working in this repository.

- Repository: <https://github.com/graelo/git-perms>
- Prefer `gh` for GitHub operations.
- Do not mention an agent or assistant in issues, pull requests, comments, or
  commit messages.
- Do not expose private local information, including machine-specific paths.

## Project

`git-perms` stores and restores Unix file permissions across git operations.
The package contains both a reusable Rust library and the `git-perms` binary,
which is also available as the `git perms` git subcommand.

Rust 1.95 or later is required. The crate uses edition 2024.

## Architecture

1. `src/bin/git-perms.rs` parses the CLI and maps commands to operations.
2. `src/git.rs` finds the repository, tracked files, git configuration, and
   hooks directory.
3. `src/perms.rs` scans, parses, serializes, applies, and compares permission
   entries.
4. `src/hooks.rs` installs and removes the automatic git hooks.
5. `src/config.rs` defines the Clap command-line configuration.
6. `src/error.rs` defines the public error type used by the library.

The `.gitperms` file is the user-facing data format. Keep its serialization
stable and preserve sorted entries. The library modules are public, but the
`git perms` command is the primary user interface.

## Verification

The `Makefile` is the canonical definition of local verification tasks. **Read
it before choosing or running verification commands**; do not duplicate its
command implementations here. `make help` lists every target.

The primary targets are:

- `make check`: pre-push gate (formatting, linting, and tests).
- `make check-all`: pre-PR gate (adds dependency, commit-message, Markdown,
  manpage, and GitHub Actions security checks).
- `make fix`: formats code and applies Clippy fixes.
- `make md`: lints Markdown against `rumdl.toml`.
- `make man`: lints the `git-perms(1)` roff manpage.
- `make ci-security`: runs the Poutine and Zizmor GitHub Actions scans.

The check targets mirror the GitHub workflows and use locked dependency
resolution where applicable. They assume their external tools (for example
`cargo-nextest`, `cargo-deny`, `cargo-pants`, `convco`, `poutine`, `zizmor`,
`rumdl`, `mandoc`, and `cargo-llvm-cov`) are already installed locally.

For focused Rust tests, use `cargo nextest run <test_name>` or
`cargo nextest run <module::tests::name>`. The complete CI test sequence is
implemented in `ci/test_full.sh`; its Nextest CI profile is configured in
`.config/nextest.toml`.

## Documentation and releases

Keep user-facing documentation in sync with behavior:

- Update `README.md` and `man/git-perms.1` when commands, output, file formats,
  hooks, or exit statuses change.
- The manpage documents the installed `git-perms` command. Lint it with
  `make man` and update its `.TH` version and date for releases.
- For a release version bump, update `Cargo.toml`, `Cargo.lock`, the versioned
  section and comparison links in `CHANGELOG.md`, and the manpage `.TH` header.
  Create a `vX.Y.Z` tag; the release workflow derives artifact and GitHub
  Release versions from it.
- Commit messages must follow `.convco` Conventional Commit rules. Use
  `make commits` to check them.

`Cargo.toml`, `Cargo.lock`, `deny.toml`, and the GitHub workflows define the
release and supply-chain constraints. Preserve `--locked` behavior in Cargo
commands that resolve dependencies.
